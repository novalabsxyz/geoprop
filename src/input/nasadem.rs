//! NASADEM evelation (`.hgt`) file format.

use crate::error::HError;
use byteorder::{LittleEndian as LE, ReadBytesExt};
use geo_types::{Coord, Polygon};
use std::{fs::File, io::ErrorKind, path::Path};

/// https://www.researchgate.net/profile/Pierre-Boulanger-4/publication/228924813/figure/fig8/AS:300852653903880@1448740270695/Description-of-a-HGT-file-structure-The-name-file-in-this-case-is-N20W100HGT.png
pub struct Hgt {
    /// Southwest corner of the tile.
    sw_corner: Coord,
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

        let sw_corner = Coord { x: 0., y: 0. };
        let cell_size = 3;
        Ok(Self {
            sw_corner,
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

fn get_lat_long<P: AsRef<Path>>(path: P) -> Result<(i32, i32), Error> {
    let stem = path.as_ref().file_stem().ok_or(Error::ParseLatLong)?;
    let desc = stem.to_str().ok_or(Error::ParseLatLong)?;
    if desc.len() != 7 {
        return Err(Error::ParseLatLong);
    }

    let get_char = |n| desc.chars().nth(n).ok_or(Error::ParseLatLong);
    let lat_sign = if get_char(0)? == 'N' { 1 } else { -1 };
    let lat: i32 = desc[1..3].parse().map_err(|_| Error::ParseLatLong)?;

    let lng_sign = if get_char(3)? == 'E' { 1 } else { -1 };
    let lng: i32 = desc[4..7].parse().map_err(|_| Error::ParseLatLong)?;
    Ok((lat_sign * lat, lng_sign * lng))
}

fn get_resolution<P: AsRef<Path>>(path: P) -> Option<Resolution> {
    let from_metadata = |m: fs::Metadata| match m.len() {
        25934402 => Some(Resolution::SRTM1),
        2884802 => Some(Resolution::SRTM3),
        _ => None,
    };
    fs::metadata(path).ok().and_then(from_metadata)
}
