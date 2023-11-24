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
pub struct IntermediateElevation {
    pub min: i16,
    pub sum: i32,
    pub max: i16,
    pub n: usize,
}

impl IntermediateElevation {
    pub fn reduce(&self) -> ReducedElevation {
        let min = self.min;
        let avg = i16::try_from(self.sum / i32::try_from(self.n).unwrap()).unwrap();
        let max = self.max;
        assert!(min <= avg && avg <= max);
        ReducedElevation { min, avg, max }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    Plain(i16),
    Intermediate(IntermediateElevation),
}

impl Elevation {
    pub fn concat(source_resolution: u8, this_resolution: u8, items: &[Self]) -> Self {
        let mut new_min = i16::MAX;
        let mut new_sum: i32 = 0;
        let mut new_max = i16::MIN;
        let mut new_n = 0_usize;
        for item in items {
            match item {
                Elevation::Plain(elev) => {
                    let n = 7_usize.pow(u32::from(source_resolution - this_resolution - 1));
                    assert_ne!(n, 0);
                    let sum = i32::from(*elev) * i32::try_from(n).unwrap();
                    new_sum += sum;
                    new_min = i16::min(new_min, *elev);
                    new_max = i16::max(new_max, *elev);
                    new_n += n;
                }

                Elevation::Intermediate(IntermediateElevation { min, sum, max, n }) => {
                    new_sum += *sum;
                    new_min = i16::min(new_min, *min);
                    new_max = i16::max(new_max, *max);
                    new_n += n;
                }
            }
        }
        Elevation::Intermediate(IntermediateElevation {
            min: new_min,
            sum: new_sum,
            max: new_max,
            n: new_n,
        })
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
            Some(Elevation::concat(
                self.source_resolution,
                cell.res(),
                &[*v0, *v1, *v2, *v3, *v4, *v5, *v6],
            ))
        } else {
            None
        }
    }
}
