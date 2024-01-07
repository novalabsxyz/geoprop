use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use hextree::{compaction::Compactor, Cell};
use std::{
    io::{Read, Write},
    mem::size_of,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elevation {
    pub min: i16,
    pub max: i16,
    pub sum: i32,
    pub n: i32,
}

impl Elevation {
    const BUF_LEN: usize =
        size_of::<i16>() + size_of::<i16>() + size_of::<i32>() + size_of::<i32>();

    pub fn new(raw: i16) -> Elevation {
        Elevation {
            min: raw,
            sum: i32::from(raw),
            max: raw,
            n: 1,
        }
    }

    pub fn from_reader<R: Read>(mut rdr: R) -> std::io::Result<Self> {
        debug_assert_eq!(Self::BUF_LEN, size_of::<Elevation>());
        let mut buf = [0_u8; Self::BUF_LEN];
        rdr.read_exact(&mut buf)?;
        let rdr = &mut &buf[..];
        let min = rdr.read_i16::<LE>()?;
        let max = rdr.read_i16::<LE>()?;
        let sum = rdr.read_i32::<LE>()?;
        let n = rdr.read_i32::<LE>()?;
        Ok(Self { min, max, sum, n })
    }

    pub fn to_writer<W: Write>(&self, mut wtr: W) -> std::io::Result<()> {
        assert_eq!(Self::BUF_LEN, size_of::<Elevation>());
        let mut buf = [0_u8; Self::BUF_LEN];
        {
            let mut buf_wtr = &mut buf[..];
            buf_wtr.write_i16::<LE>(self.min)?;
            buf_wtr.write_i16::<LE>(self.max)?;
            buf_wtr.write_i32::<LE>(self.sum)?;
            buf_wtr.write_i32::<LE>(self.n)?;
        }
        wtr.write_all(&buf)
    }
}

impl Elevation {
    pub fn concat(items: &[&Self]) -> Self {
        let mut min = i16::MAX;
        let mut sum: i32 = 0;
        let mut max = i16::MIN;
        let mut n = 0_i32;
        for item in items {
            sum += item.sum;
            min = i16::min(min, item.min);
            max = i16::max(max, item.max);
            n += item.n;
        }
        Elevation { min, max, sum, n }
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
            Some(Elevation::concat(&[v0, v1, v2, v3, v4, v5, v6]))
        } else {
            None
        }
    }
}

pub struct CloseEnoughCompactor {
    // Maximum differance between min and max child elevations
    // allowable for a cell to be coalesced.
    pub tolerance: i16,
}

impl Compactor<Elevation> for CloseEnoughCompactor {
    fn compact(&mut self, _cell: Cell, children: [Option<&Elevation>; 7]) -> Option<Elevation> {
        if let [Some(v0), Some(v1), Some(v2), Some(v3), Some(v4), Some(v5), Some(v6)] = children {
            let mut n_min = i16::MAX;
            let mut n_sum = 0;
            let mut n_max = i16::MIN;
            let mut n_n = 0;
            for Elevation { min, sum, max, n } in [v0, v1, v2, v3, v4, v5, v6] {
                // HACK: Ignore voids that snuck through.
                if [min, max].contains(&&i16::MIN) {
                    continue;
                }
                n_min = i16::min(n_min, *min);
                n_sum += sum;
                n_max = i16::max(n_max, *max);
                n_n += n;
            }
            let error = n_max - n_min;
            assert!(error >= 0, "error can't be negative");
            if error <= self.tolerance {
                Some(Elevation {
                    min: n_min,
                    sum: n_sum,
                    max: n_max,
                    n: n_n,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
