use anyhow::Result;
use byteorder::{LittleEndian as LE, ReadBytesExt};
use hextree::{compaction::Compactor, Cell};
use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReducedElevation {
    pub min: i16,
    pub avg: i16,
    pub max: i16,
}

impl ReducedElevation {
    pub fn from_reader<R: Read>(mut rdr: R) -> Result<Self> {
        let mut buf = [0_u8; 3 * std::mem::size_of::<i16>()];
        rdr.read_exact(&mut buf)?;
        let rdr = &mut &buf[..];
        let min = rdr.read_i16::<LE>()?;
        let avg = rdr.read_i16::<LE>()?;
        let max = rdr.read_i16::<LE>()?;
        Ok(Self { min, avg, max })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elevation {
    pub min: i16,
    pub sum: i32,
    pub max: i16,
    pub n: usize,
}

impl Elevation {
    pub fn new(raw: i16) -> Elevation {
        Elevation {
            min: raw,
            sum: i32::from(raw),
            max: raw,
            n: 1,
        }
    }

    pub fn reduce(&self) -> ReducedElevation {
        let min = self.min;
        let avg = i16::try_from(self.sum / i32::try_from(self.n).unwrap()).unwrap();
        let max = self.max;
        assert_ne!(min, i16::MIN);
        assert_ne!(min, i16::MAX);
        assert_ne!(max, i16::MIN);
        assert_ne!(max, i16::MAX);
        assert!(min <= avg && avg <= max);
        ReducedElevation { min, avg, max }
    }
}

impl Elevation {
    pub fn concat(items: &[Self]) -> Self {
        let mut min = i16::MAX;
        let mut sum: i32 = 0;
        let mut max = i16::MIN;
        let mut n = 0_usize;
        for item in items {
            sum += item.sum;
            min = i16::min(min, item.min);
            max = i16::max(max, item.max);
            n += item.n;
        }
        Elevation { min, sum, max, n }
    }
}

pub struct ReductionCompactor {
    pub target_resolution: u8,
    pub source_resolution: u8,
}

impl Compactor<Elevation> for ReductionCompactor {
    fn compact(&mut self, cell: Cell, children: [Option<&Elevation>; 7]) -> Option<Elevation> {
        if cell.res() < self.target_resolution {
            None
        } else if let [Some(v0), Some(v1), Some(v2), Some(v3), Some(v4), Some(v5), Some(v6)] =
            children
        {
            Some(Elevation::concat(&[*v0, *v1, *v2, *v3, *v4, *v5, *v6]))
        } else {
            None
        }
    }
}
