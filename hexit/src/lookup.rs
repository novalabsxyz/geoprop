use crate::options::Lookup;
use anyhow::Result;
use byteorder::{LittleEndian as LE, ReadBytesExt};
use hextree::{disktree::DiskTree, Cell};

impl Lookup {
    pub fn run(&self) -> Result<()> {
        let raw_cell: u64 = self
            .cell
            .parse::<u64>()
            .or_else(|_| u64::from_str_radix(&self.cell, 16))?;
        let cell = Cell::try_from(raw_cell)?;
        let mut disktree = DiskTree::open(&self.disktree)?;
        match disktree.seek_to_cell(cell)? {
            None => (),
            Some((cell, rdr)) => {
                let val = rdr.read_i16::<LE>()?;
                println!("{cell}: {val}");
            }
        }
        Ok(())
    }
}
