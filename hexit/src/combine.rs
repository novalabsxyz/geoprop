use crate::{options::Combine, progress};
use anyhow::Result;
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use flate2::bufread::GzDecoder;
use hextree::{compaction::EqCompactor, disktree::DiskTree, Cell, HexTreeMap};
use indicatif::MultiProgress;
use std::{ffi::OsStr, fs::File, io::BufReader, path::Path};

impl Combine {
    pub fn run(&self) -> Result<()> {
        assert!(!self.input.is_empty());
        let mut hextree: HexTreeMap<i16, EqCompactor> = HexTreeMap::with_compactor(EqCompactor);
        let progress_group = MultiProgress::new();
        for tess_file_path in &self.input {
            Self::read_tessellation(tess_file_path, &progress_group, &mut hextree)?;
        }
        self.write_disktree(&hextree, &progress_group)?;
        self.verify_disktree(&hextree, &progress_group)?;
        Ok(())
    }

    fn read_tessellation(
        tess_file_path: &Path,
        progress_group: &MultiProgress,
        hextree: &mut HexTreeMap<i16, EqCompactor>,
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
            hextree.insert(cell, elevation);
            pb.inc(1);
        }
        assert!(
            rdr.read_u8().is_err(),
            "We should have read all samples out of the file"
        );

        Ok(())
    }

    fn write_disktree(
        &self,
        hextree: &HexTreeMap<i16, EqCompactor>,
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
        hextree.to_disktree(disktree_file, |wtr, val| {
            pb.inc(1);
            wtr.write_i16::<LE>(*val)
        })?;
        Ok(())
    }

    fn verify_disktree(
        &self,
        hextree: &HexTreeMap<i16, EqCompactor>,
        progress_group: &MultiProgress,
    ) -> Result<()> {
        fn value_reader(res: hextree::Result<(Cell, &mut File)>) -> Result<(Cell, i16)> {
            let (cell, rdr) = res?;
            Ok(rdr.read_i16::<LE>().map(|val| (cell, val))?)
        }

        let mut disktree = DiskTree::open(&self.out)?;
        let disktree_file_name = self
            .out
            .file_name()
            .and_then(OsStr::to_str)
            .expect("already opened, therefore path must be a file");
        let pb = progress_group.add(progress::bar(
            format!("Validating {disktree_file_name}"),
            hextree.len() as u64,
        ));
        let mut count = 0;
        for res in disktree.iter(value_reader)? {
            let (cell, value) = res?;
            assert_eq!(Some((cell, &value)), hextree.get(cell));
            pb.inc(1);
            count += 1;
        }
        assert_eq!(hextree.len(), count);
        Ok(())
    }
}
