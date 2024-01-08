use crate::{elevation::Elevation, mask, options::Json};
use anyhow::Result;
use geo::geometry::GeometryCollection;
use h3o::{
    geom::{PolyfillConfig, ToCells},
    Resolution,
};
use hextree::{disktree::DiskTreeMap, memmap::Mmap, Cell, HexTreeMap};
use serde::Serialize;
use serde_json::Value;

impl Json {
    pub fn run(&self) -> Result<()> {
        let disktree = DiskTreeMap::open(&self.disktree)?;
        let mask = mask::open(Some(&self.mask))?.unwrap();
        let target_cells = Self::polyfill_mask(mask, self.resolution)?;
        let mut hextree = HexTreeMap::new();
        for h3idx in target_cells {
            let cell = Cell::try_from(h3idx)?;
            if let Some((cell, reduction)) = Self::get(cell, &disktree)? {
                hextree.insert(cell, reduction);
            }
        }
        let json = Self::gen_json(&hextree)?;
        Self::output_json(&json)?;
        Ok(())
    }

    fn polyfill_mask(mask: GeometryCollection, resolution: Resolution) -> Result<Vec<u64>> {
        let polygon = h3o::geom::GeometryCollection::from_degrees(mask)?;
        let mut cells: Vec<u64> = polygon
            .to_cells(PolyfillConfig::new(resolution))
            .map(u64::from)
            .collect();
        cells.sort_unstable();
        cells.dedup();
        Ok(cells)
    }

    fn get(cell: Cell, disktree: &DiskTreeMap<Mmap>) -> Result<Option<(Cell, Elevation)>> {
        match disktree.get(cell)? {
            None => Ok(None),
            Some((cell, bytes)) => {
                let reduction = Elevation::from_reader(&mut &bytes[..])?;
                Ok(Some((cell, reduction)))
            }
        }
    }

    fn gen_json(hextree: &HexTreeMap<Elevation>) -> Result<Value> {
        #[derive(Serialize)]
        struct JsonEntry {
            h3_id: String,
            min: i16,
            avg: i16,
            sum: i32,
            max: i16,
            n: i32,
        }
        impl From<(Cell, &Elevation)> for JsonEntry {
            fn from((cell, elev): (Cell, &Elevation)) -> JsonEntry {
                JsonEntry {
                    avg: i16::try_from(elev.sum / elev.n).unwrap(),
                    h3_id: cell.to_string(),
                    max: elev.max,
                    min: elev.min,
                    n: elev.n,
                    sum: elev.sum,
                }
            }
        }
        let samples = hextree
            .iter()
            .map(JsonEntry::from)
            .map(serde_json::to_value)
            .collect::<Result<Vec<Value>, _>>()?;
        Ok(Value::Array(samples))
    }

    fn output_json(json: &Value) -> Result<()> {
        let out = std::io::stdout().lock();
        serde_json::to_writer(out, json)?;
        Ok(())
    }
}
