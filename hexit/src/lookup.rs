use crate::{elevation::Elevation, options::Lookup};
use anyhow::Result;
use hextree::{disktree::DiskTreeMap, memmap::Mmap, Cell};

impl Lookup {
    pub fn run(&self) -> Result<()> {
        let raw_cell: u64 = self
            .cell
            .parse::<u64>()
            .or_else(|_| u64::from_str_radix(&self.cell, 16))?;
        let cell = Cell::try_from(raw_cell)?;
        let mut disktree = DiskTreeMap::open(&self.disktree)?;

        Self::by_get(cell, &mut disktree)
    }

    fn by_get(cell: Cell, disktree: &mut DiskTreeMap<Mmap>) -> Result<()> {
        let t0 = std::time::Instant::now();
        match disktree.get(cell)? {
            None => (),
            Some((cell, bytes)) => {
                let t_seek = t0.elapsed();
                let Elevation { min, max, sum, n } = Elevation::from_reader(&mut &bytes[..])?;
                let avg = sum / n;
                println!("cell: {cell} (res {})", cell.res());
                println!("min:  {min}");
                println!("avg:  {avg}");
                println!("max:  {max}");
                println!("n:    {n}");
                println!("seek: {t_seek:?}");
            }
        }
        Ok(())
    }
}
