use crate::{
    elevation::{Elevation, ReducedElevation, ReductionCompactor},
    options::Combine,
    progress,
};
use anyhow::Result;
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use flate2::bufread::GzDecoder;
use hextree::HexTreeMap;
use indicatif::MultiProgress;
use std::{ffi::OsStr, fs::File, io::BufReader, path::Path};

impl Combine {
    pub fn run(&self) -> Result<()> {
        assert!(!self.input.is_empty());
        let mut hextree: HexTreeMap<Elevation, ReductionCompactor> =
            HexTreeMap::with_compactor(ReductionCompactor {
                source_resolution: self.source_resolution as u8,
                target_resolution: self.target_resolution as u8,
            });
        let progress_group = MultiProgress::new();
        for tess_file_path in &self.input {
            Self::read_tessellation(tess_file_path, &progress_group, &mut hextree)?;
        }
        let hextree = self.reduce_hextree(&hextree, &progress_group);
        self.write_disktree(&hextree, &progress_group)?;
        Ok(())
    }

    fn read_tessellation(
        tess_file_path: &Path,
        progress_group: &MultiProgress,
        hextree: &mut HexTreeMap<Elevation, ReductionCompactor>,
    ) -> Result<()> {
        let tess_file = File::open(tess_file_path)?;
        let tess_buf_rdr = BufReader::new(tess_file);
        let mut rdr = GzDecoder::new(tess_buf_rdr);
        let tess_file_name = tess_file_path
            .file_name()
            .and_then(OsStr::to_str)
            .expect("already opened, therefore path must be a file");

        let n_samples = rdr.read_u64::<LE>()?;
        let pb = progress_group.add(progress::bar(tess_file_name.to_string(), n_samples));
        for _sample_n in 0..n_samples {
            let raw_cell = rdr.read_u64::<LE>()?;
            let cell = hextree::Cell::from_raw(raw_cell)?;
            let elevation = rdr.read_i16::<LE>()?;
            hextree.insert(cell, Elevation::Plain(elevation));
            pb.inc(1);
        }
        assert!(
            rdr.read_u8().is_err(),
            "We should have read all samples out of the file"
        );

        Ok(())
    }

    fn reduce_hextree(
        &self,
        hextree: &HexTreeMap<Elevation, ReductionCompactor>,
        _progress_group: &MultiProgress,
    ) -> HexTreeMap<ReducedElevation> {
        let mut reduced_hextree = HexTreeMap::new();
        let max_child_cnt =
            7_usize.pow(self.source_resolution as u32 - self.target_resolution as u32);
        for (cell, elev) in hextree.iter() {
            match elev {
                Elevation::Intermediate(intermediate)
                    if cell.res() == self.target_resolution as u8 =>
                {
                    assert_eq!(intermediate.n, max_child_cnt);
                    let reduction = intermediate.reduce();
                    reduced_hextree.insert(cell, reduction);
                }
                _ => {}
            };
        }
        reduced_hextree
    }

    fn write_disktree(
        &self,
        hextree: &HexTreeMap<ReducedElevation>,
        progress_group: &MultiProgress,
    ) -> Result<()> {
        let disktree_file = File::create(&self.out)?;
        let disktree_file_name = self
            .out
            .file_name()
            .and_then(OsStr::to_str)
            .expect("already opened, therefore path must be a file");
        let disktree_len = hextree.len();
        let pb = progress_group.add(progress::bar(
            format!("Writing {disktree_file_name}"),
            disktree_len as u64,
        ));
        hextree.to_disktree(disktree_file, |wtr, ReducedElevation { min, avg, max }| {
            pb.inc(1);
            wtr.write_i16::<LE>(*min)
                .and_then(|()| wtr.write_i16::<LE>(*avg))
                .and_then(|()| wtr.write_i16::<LE>(*max))
        })?;
        Ok(())
    }
}
