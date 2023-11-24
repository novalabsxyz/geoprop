use crate::{elevation::ReducedElevation, options::Lookup};
use anyhow::Result;
use hextree::{disktree::DiskTree, Cell};
use std::fs::File;

impl Lookup {
    pub fn run(&self) -> Result<()> {
        let raw_cell: u64 = self
            .cell
            .parse::<u64>()
            .or_else(|_| u64::from_str_radix(&self.cell, 16))?;
        let cell = Cell::try_from(raw_cell)?;
        let mut disktree = DiskTree::open(&self.disktree)?;

        Self::by_get(cell, &mut disktree)
    }

    fn by_get(cell: Cell, disktree: &mut DiskTree<File>) -> Result<()> {
        let t0 = std::time::Instant::now();
        match disktree.seek_to_cell(cell)? {
            None => (),
            Some((cell, rdr)) => {
                let t_seek = t0.elapsed();
                let ReducedElevation { min, avg, max } = ReducedElevation::from_reader(rdr)?;
                let t_tot = t0.elapsed();
                println!("cell: {cell} (res {})", cell.res());
                println!("min:  {min}");
                println!("avg:  {avg}");
                println!("max:  {max}");
                println!("seek: {t_seek:?}");
            }
        }
        Ok(())
    }
}
