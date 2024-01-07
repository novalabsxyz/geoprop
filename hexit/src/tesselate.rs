use crate::{mask, options::Tesselate, progress};
use anyhow::Result;
use byteorder::{LittleEndian as LE, WriteBytesExt};
use flate2::{write::GzEncoder, Compression};
use geo::{GeometryCollection, Intersects};
use h3o::{
    geom::{PolyfillConfig, Polygon, ToCells},
    Resolution,
};
use hextree::{Cell, HexTreeMap};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
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
        let progress_group = MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(4));
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
        let (in_file_name, out_file_name) = {
            let file_name = height_file_path
                .file_name()
                .and_then(OsStr::to_str)
                .expect("already opened, therefore path must be a file");
            (
                file_name,
                format!("{file_name}.res{}.h3tez", self.resolution),
            )
        };
        let out_file_path = self.out_dir.clone().join(&out_file_name);

        if out_file_path.exists() && !self.overwrite {
            return Ok(());
        }

        let out_file_tmp_path = {
            let mut p = out_file_path.clone();
            p.set_extension("tmp");
            p
        };

        let tile = Tile::memmap(height_file_path)?;
        let intersects = mask.as_ref().map_or(true, |mask| {
            let polygon = tile.polygon();
            mask.intersects(&polygon)
        });
        if intersects {
            let pb = progress_group.add(progress::bar(
                format!("Tesselate {in_file_name}"),
                tile.len() as u64,
            ));
            let hextree = self.tesselate_tile(&tile, &pb)?;
            let tmp_out_file = File::create(&out_file_tmp_path)?;
            let tmp_out_wtr = GzEncoder::new(tmp_out_file, Compression::new(self.compression));
            let wtr = BufWriter::new(tmp_out_wtr);
            let pb = progress_group.add(progress::bar(
                format!("Write {out_file_name}"),
                hextree.len() as u64,
            ));
            Self::write_to_disk(&hextree, &pb, wtr)?;
            fs::rename(out_file_tmp_path, out_file_path)?;
        }

        Ok(())
    }

    fn tesselate_tile(&self, tile: &Tile, progress_bar: &ProgressBar) -> Result<HexTreeMap<i16>> {
        let mut hextree: HexTreeMap<i16> = HexTreeMap::new();
        for sample in tile.iter() {
            assert_ne!(sample.elevation(), i16::MIN);
            let (elev, hexes) = Self::tesselate_sample(&sample, self.resolution)?;
            for hex in hexes {
                hextree.insert(Cell::try_from(hex)?, elev);
            }
            progress_bar.inc(1);
        }
        Ok(hextree)
    }

    fn tesselate_sample(sample: &Sample, resolution: Resolution) -> Result<(i16, Vec<u64>)> {
        let elevation = sample.elevation();
        let polygon = Polygon::from_degrees(sample.polygon())?;
        let mut cells: Vec<u64> = polygon
            .to_cells(PolyfillConfig::new(resolution))
            .map(u64::from)
            .collect();
        cells.sort_unstable();
        cells.dedup();
        Ok((elevation, cells))
    }

    fn write_to_disk(
        hextree: &HexTreeMap<i16>,
        progress_bar: &ProgressBar,
        mut out: impl Write,
    ) -> Result<()> {
        out.write_u64::<LE>(hextree.len() as u64)?;
        for (cell, elev) in hextree.iter() {
            out.write_u64::<LE>(cell.into_raw())?;
            out.write_i16::<LE>(*elev)?;
            progress_bar.inc(1);
        }
        Ok(())
    }
}
