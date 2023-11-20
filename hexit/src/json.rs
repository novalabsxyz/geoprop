use crate::{elevation::ReducedElevation, mask, options::Json};
use anyhow::Result;
use geo::geometry::GeometryCollection;
use h3o::{
    geom::{PolyfillConfig, ToCells},
    Resolution,
};
use hextree::{disktree::DiskTree, Cell, HexTreeMap};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::File;

impl Json {
    pub fn run(&self) -> Result<()> {
        let mut disktree = DiskTree::open(&self.disktree)?;
        let mask = mask::open(Some(&self.mask))?.unwrap();
        let target_cells = Self::polyfill_mask(mask, self.source_resolution)?;
        let mut hextree = HexTreeMap::new();
        for h3idx in target_cells {
            let cell = Cell::try_from(h3idx)?;
            if let Some((cell, reduction)) = Self::get(cell, &mut disktree)? {
                hextree.insert(cell, reduction);
            }
        }
        let json = Self::gen_json(&hextree);
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

    fn get(cell: Cell, disktree: &mut DiskTree<File>) -> Result<Option<(Cell, ReducedElevation)>> {
        match disktree.seek_to_cell(cell)? {
            None => Ok(None),
            Some((cell, rdr)) => {
                let reduction = ReducedElevation::from_reader(rdr)?;
                Ok(Some((cell, reduction)))
            }
        }
    }

    fn gen_json(hextree: &HexTreeMap<ReducedElevation>) -> Value {
        #[derive(Serialize)]
        struct JsonEntry {
            h3_id: String,
            min: i16,
            avg: i16,
            max: i16,
        }
        impl From<(Cell, &ReducedElevation)> for JsonEntry {
            fn from((cell, reduction): (Cell, &ReducedElevation)) -> JsonEntry {
                JsonEntry {
                    h3_id: cell.to_string(),
                    min: reduction.min,
                    avg: reduction.avg,
                    max: reduction.max,
                }
            }
        }
        let samples = hextree.iter().map(JsonEntry::from).collect::<Vec<_>>();
        json!(samples)
    }

    fn output_json(json: &Value) -> Result<()> {
        let out = std::io::stdout();
        serde_json::to_writer(out, json)?;
        Ok(())
    }
}
