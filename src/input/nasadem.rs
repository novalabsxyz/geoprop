//! NASADEM evelation (`.hgt`) file format.
//!
//! https://dwtkns.com/srtm30m
//! https://www.researchgate.net/profile/Pierre-Boulanger-4/publication/228924813/figure/fig8/AS:300852653903880@1448740270695/Description-of-a-HGT-file-structure-The-name-file-in-this-case-is-N20W100HGT.png

use crate::error::HError;
use byteorder::{LittleEndian as LE, ReadBytesExt};
use geo_types::{Coord, Polygon};
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    path::Path,
};

pub struct Tile {
    /// Southwest corner of the tile.
    sw_corner: Coord<i16>,

    /// Arcseconds per sample.
    resolution: u8,

    /// Number of (rows, columns) in this tile.
    dimensions: (usize, usize),

    /// Elevation samples.
    samples: Box<[i16]>,
}

impl Tile {
    /// Returnes Self parsed from the file at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HError> {
        let sw_corner = parse_sw_corner(&path)?;

        let (resolution, dimensions) = extract_resolution(&path)?;
        let mut file = BufReader::new(File::open(path)?);

        let samples = {
            let mut samples = Vec::new();

            for row in 0..dimensions.1 {
                file.seek(SeekFrom::End(
                    -((((row + 1) * dimensions.0) * std::mem::size_of::<i16>()) as i64),
                ))?;
                for _col in 0..dimensions.0 {
                    let sample = file.read_i16::<LE>()?;
                    samples.push(sample);
                }
            }
            assert_eq!(samples.len(), dimensions.0 * dimensions.1);
            samples.into_boxed_slice()
        };

        Ok(Self {
            sw_corner,
            resolution,
            dimensions,
            samples,
        })
    }

    /// Rreturns this tile's resolution in arcseconds per sample.
    pub fn resolution(&self) -> u8 {
        self.resolution
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
    pub fn iter(&self) -> impl Iterator<Item = Sample<'_>> + '_ {
        (0..self.samples.len()).map(|index| Sample {
            parent: self,
            index,
        })
    }
}

impl std::ops::Index<Coord<f64>> for Tile {
    type Output = i16;

    fn index(&self, _coord: Coord<f64>) -> &Self::Output {
        unimplemented!()
    }
}

impl std::ops::Index<(usize, usize)> for Tile {
    type Output = i16;

    fn index(&self, _idx: (usize, usize)) -> &Self::Output {
        unimplemented!()
    }
}

/// A NASADEM elevation sample.
pub struct Sample<'a> {
    /// The parent [Hgt] this grid square belongs to.
    parent: &'a Tile,
    /// Index into parent's evelation data corresponding to tbhis grid
    /// square.
    index: usize,
}

impl<'a> Sample<'a> {
    pub fn elevation(&self) -> i16 {
        self.parent.samples[self.index]
    }

    pub fn polygon(&self) -> Polygon {
        unimplemented!()
    }
}

fn extract_resolution<P: AsRef<Path>>(path: P) -> Result<(u8, (usize, usize)), HError> {
    const RES_1_ARCSECONDS_FILE_LEN: u64 = 3601 * 3601 * std::mem::size_of::<u16>() as u64;
    const RES_3_ARCSECONDS_FILE_LEN: u64 = 1201 * 1201 * std::mem::size_of::<u16>() as u64;
    match path.as_ref().metadata().map(|m| m.len())? {
        RES_1_ARCSECONDS_FILE_LEN => Ok((1, (3601, 3601))),
        RES_3_ARCSECONDS_FILE_LEN => Ok((3, (1201, 1201))),
        invalid_len => Err(HError::HgtLen(invalid_len)),
    }
}

fn parse_sw_corner<P: AsRef<Path>>(path: P) -> Result<Coord<i16>, HError> {
    let mk_err = || HError::HgtName(path.as_ref().to_owned());
    let name = path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(mk_err)?;
    if name.len() != 7 {
        return Err(mk_err());
    }
    let lat_sign = match &name[0..1] {
        "N" => 1,
        "S" => -1,
        _ => return Err(mk_err()),
    };
    let lat = lat_sign * name[1..3].parse::<i16>().map_err(|_| mk_err())?;
    let lon_sign = match &name[3..4] {
        "E" => 1,
        "W" => -1,
        _ => return Err(mk_err()),
    };
    let lon = lon_sign * name[4..7].parse::<i16>().map_err(|_| mk_err())?;
    Ok(Coord { x: lon, y: lat })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn one_arcsecond_dir() -> PathBuf {
        [env!("CARGO_MANIFEST_DIR"), "data", "nasadem", "1arcsecond"]
            .iter()
            .collect()
    }

    #[test]
    fn test_parse_hgt_name() {
        let mut path = one_arcsecond_dir();
        path.push("N44W072.hgt");
        let sw_corner = parse_sw_corner(&path).unwrap();
        let resolution = extract_resolution(&path).unwrap();
        assert_eq!(sw_corner, Coord { x: -72, y: 44 });
        assert_eq!(resolution, (1, (3601, 3601)));
    }

    #[test]
    fn test_tile_open() {
        let mut path = one_arcsecond_dir();
        path.push("N44W072.hgt");
        Tile::open(path).unwrap();
    }
}
