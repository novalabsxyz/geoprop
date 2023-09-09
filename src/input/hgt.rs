//! NASADEM evelation (`.hgt`) file format.

use crate::error::HError;
use byteorder::{LittleEndian as LE, ReadBytesExt};
use geo_types::{polygon, Coord, Polygon};
use std::{fs::File, io::ErrorKind, path::Path};

pub struct Hgt {
    /// Lower-left corner of the file.
    ll_corner: Coord,
    /// Arcseconds per sample.
    cell_size: u8,
    /// Elevation samples.
    samples: Vec<i16>,
}

impl Hgt {
    /// Returnes Self parsed from the file at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HError> {
        let mut file = File::open(path)?;
        let mut samples = Vec::new();

        loop {
            match file.read_i16::<LE>() {
                Ok(sample) => samples.push(sample),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(HError::Io(e)),
            };
        }

        let ll_corner = Coord { x: 0., y: 0. };
        let cell_size = 3;
        Ok(Self {
            ll_corner,
            cell_size,
            samples,
        })
    }

    /// Returns the sample at `[x, y]`.
    pub fn sample_at_idx(&self, _x: usize, _y: usize) -> i16 {
        unimplemented!()
    }

    /// Returns the sample at the given geo coordinates.
    pub fn sample_at_coord(&self, _coord: Coord) -> i16 {
        unimplemented!()
    }

    pub fn poly_at_idx(&self, idx: usize) -> Polygon<f64> {
        unimplemented!()
    }

    /// Returns and iterator over `self`'s grid squares.
    pub fn iter(&self) -> impl Iterator<Item = GridSquare<'_>> + '_ {
        (0..self.samples.len()).map(|index| GridSquare {
            parent: self,
            index,
        })
    }
}

/// A NASADEM elevation sample.
pub struct GridSquare<'a> {
    /// The parent [Hgt] this grid square belongs to.
    parent: &'a Hgt,
    /// Index into parent's evelation data corresponding to tbhis grid
    /// square.
    index: usize,
}

impl<'a> GridSquare<'a> {
    pub fn elevation(&self) -> i16 {
        self.parent.samples[self.index]
    }

    pub fn polygon(&self) -> Polygon {
        unimplemented!()
    }
}
