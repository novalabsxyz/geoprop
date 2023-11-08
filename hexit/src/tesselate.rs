use crate::{mask, options::Tesselate, progress};
use anyhow::Result;
use byteorder::{LittleEndian as LE, WriteBytesExt};
use flate2::{write::GzEncoder, Compression};
use geo::{GeometryCollection, Intersects};
use h3o::{
    geom::{PolyfillConfig, Polygon, ToCells},
    CellIndex, Resolution,
};
use indicatif::{MultiProgress, ProgressBar};
use nasadem::{Sample, Tile};
use rayon::prelude::*;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

impl Tesselate {
    pub fn run(&self) -> Result<()> {
        let progress_group = MultiProgress::new();
        let mask = mask::open(self.mask.as_deref())?;
        self.input.par_iter().try_for_each(|height_file_path| {
            self._run(height_file_path, mask.as_ref(), &progress_group)
        })?;
        Ok(())
    }

    fn _run(
        &self,
        height_file_path: &Path,
        mask: Option<&GeometryCollection>,
        progress_group: &MultiProgress,
    ) -> Result<()> {
        let out_file_name = {
            let file_name = height_file_path
                .file_name()
                .and_then(OsStr::to_str)
                .expect("already opened, therefore path must be a file");
            format!("{file_name}.res{}.h3tez", self.resolution)
        };
        let out_file_path = self.out_dir.clone().join(&out_file_name);

        if out_file_path.exists() && !self.overwrite {
            // Exit early if we've already processed this input.
            return Ok(());
        }

        let out_file_tmp_path = {
            let mut p = out_file_path.clone();
            p.set_extension("tmp");
            p
        };

        let tile = Tile::memmap(height_file_path)?;
        let intersects = mask
            .as_ref()
            .map_or(true, |mask| mask.intersects(&tile.polygon()));
        if intersects {
            let pb = progress_group.add(progress::bar(out_file_name, tile.len() as u64));
            let tmp_out_file = File::create(&out_file_tmp_path)?;
            let tmp_out_wtr = GzEncoder::new(tmp_out_file, Compression::new(self.compression));
            self.polyfill_tile(&tile, &pb, BufWriter::new(tmp_out_wtr))?;
            fs::rename(out_file_tmp_path, out_file_path)?;
        }

        Ok(())
    }

    fn polyfill_tile(
        &self,
        tile: &Tile,
        progress_bar: &ProgressBar,
        mut out: impl Write,
    ) -> Result<()> {
        out.write_u64::<LE>(tile.len() as u64)?;
        for sample in tile.iter() {
            let (elev, hexes) = polyfill_sample(&sample, self.resolution)?;
            out.write_i16::<LE>(elev)?;
            out.write_u16::<LE>(u16::try_from(hexes.len())?)?;
            for hex in hexes {
                out.write_u64::<LE>(hex)?;
            }
            progress_bar.inc(1);
        }
        Ok(())
    }
}

fn polyfill_sample(sample: &Sample, resolution: Resolution) -> Result<(i16, Vec<u64>)> {
    let elevation = sample.elevation();
    let polygon = Polygon::from_degrees(sample.polygon())?;
    let cell_iter = polygon.to_cells(PolyfillConfig::new(resolution));
    let mut cells: Vec<u64> = CellIndex::compact(cell_iter)?.map(u64::from).collect();
    cells.sort_unstable();
    cells.dedup();
    Ok((elevation, cells))
}
