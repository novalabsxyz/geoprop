use crate::options::Lookup;
use anyhow::Result;
use byteorder::{LittleEndian as LE, ReadBytesExt};
use hextree::{disktree::DiskTree, Cell};
use std::{fs::File, io::Write};

impl Lookup {
    pub fn run(&self) -> Result<()> {
        let raw_cell: u64 = self
            .cell
            .parse::<u64>()
            .or_else(|_| u64::from_str_radix(&self.cell, 16))?;
        let cell = Cell::try_from(raw_cell)?;
        let mut disktree = DiskTree::open(&self.disktree)?;

        if self.iter {
            Self::by_iter(cell, &mut disktree)
        } else {
            Self::by_get(cell, &mut disktree)
        }
    }

    fn by_get(cell: Cell, disktree: &mut DiskTree<File>) -> Result<()> {
        let t0 = std::time::Instant::now();
        match disktree.seek_to_cell(cell)? {
            None => (),
            Some((cell, rdr)) => {
                let t_seek = t0.elapsed();
                let elev = rdr.read_i16::<LE>()?;
                let t_tot = t0.elapsed();
                println!("{cell}: {elev}");
                println!("{t_seek:?} {t_tot:?}");
            }
        }
        Ok(())
    }

    fn by_iter(_target_cell: Cell, disktree: &mut DiskTree<File>) -> Result<()> {
        fn read_elev(res: hextree::Result<(Cell, &mut File)>) -> Result<Option<(Cell, i16)>> {
            let (cell, rdr) = res?;
            let mask = Cell::try_from(0x8126bffffffffff)?;
            if cell.is_related_to(&mask) {
                Ok(Some((cell, rdr.read_i16::<LE>()?)))
            } else {
                Ok(None)
            }
        }
        let mut stderr = std::io::stderr().lock();
        for res in disktree.iter(read_elev)? {
            if let Some((cell, elev)) = res? {
                writeln!(&mut stderr, "{cell}: {elev}")?;
            }
        }
        Ok(())
    }
}
