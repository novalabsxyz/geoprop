#![allow(dead_code)]

//! NASADEM evelation (`.hgt`) file format.
//!
//! # References
//!
//! 1. [30-Meter SRTM Tile Downloader](https://dwtkns.com/srtm30m)
//! 1. [HGT file layout](https://www.researchgate.net/profile/Pierre-Boulanger-4/publication/228924813/figure/fig8/AS:300852653903880@1448740270695/Description-of-a-HGT-file-structure-The-name-file-in-this-case-is-N20W100HGT.png)
//! 1. [Archive Team](http://fileformats.archiveteam.org/index.php?title=HGT&oldid=17250)
//! 1. [SRTM Collection User Guide](https://lpdaac.usgs.gov/documents/179/SRTM_User_Guide_V3.pdf)

mod error;

pub use crate::error::HError;
use byteorder::{BigEndian as BE, ReadBytesExt};
use geo_types::{polygon, Coord, Polygon};
#[cfg(feature = "kml")]
use kml::{
    self,
    types::{Coord as KmlCoord, LinearRing as KmlLinearRing, Polygon as KmlPolygon},
    Kml,
};
use memmap2::Mmap;
use std::{fs::File, io::BufReader, mem::size_of, path::Path};

const ARCSEC_PER_DEG: f64 = 3600.0;
const HALF_ARCSEC: f64 = 1.0 / (2.0 * 3600.0);

pub struct Tile {
    /// Southwest corner of the tile.
    ///
    /// Specificlly, the _center_ of the SW most sample of the tile.
    sw_corner: Coord<f64>,

    /// Northeast corner of the tile.
    ///
    /// Specificlly, the _center_ of the NE most sample of the tile.
    ne_corner: Coord<f64>,

    /// Arcseconds per sample.
    resolution: u8,

    /// Number of (rows, columns) in this tile.
    dimensions: (usize, usize),

    /// Lowest elevation sample in this tile.
    min_elev: i16,

    /// Highest elevation sample in this tile.
    max_elev: i16,

    /// Elevation samples.
    samples: Storage,
}

enum Storage {
    Parsed(Box<[i16]>),
    Mapped(Mmap),
}

impl Storage {
    fn get_unchecked(&self, index: usize) -> i16 {
        match self {
            Storage::Parsed(samples) => samples[index],
            Storage::Mapped(raw) => {
                let start = index * size_of::<u16>();
                let end = start + size_of::<u16>();
                let bytes = &mut &raw.as_ref()[start..end];
                bytes.read_i16::<BE>().unwrap()
            }
        }
    }

    /// Returns the lowest elevation sample in this data.
    fn min_elev(&self) -> i16 {
        match self {
            Storage::Parsed(samples) => samples.iter().max().copied().unwrap(),
            Storage::Mapped(raw) => (*raw)
                .chunks_exact(2)
                .map(|mut bytes| (&mut bytes).read_i16::<BE>().unwrap())
                .max()
                .unwrap(),
        }
    }

    /// Returns the highest elevation sample in this data.
    pub fn max_elev(&self) -> i16 {
        match self {
            Storage::Parsed(samples) => samples.iter().max().copied().unwrap(),
            Storage::Mapped(raw) => (*raw)
                .chunks_exact(2)
                .map(|mut bytes| (&mut bytes).read_i16::<BE>().unwrap())
                .max()
                .unwrap(),
        }
    }
}

impl Tile {
    /// Returns Self parsed from the file at `path`.
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self, HError> {
        let (resolution, dimensions @ (cols, rows)) = extract_resolution(&path)?;
        let sw_corner = {
            let Coord { x, y } = parse_sw_corner(&path)?;
            Coord {
                x: f64::from(x),
                y: f64::from(y),
            }
        };

        let ne_corner = Coord {
            y: sw_corner.y + (dimensions.0 as f64 * f64::from(resolution)) / ARCSEC_PER_DEG,
            x: sw_corner.x + (dimensions.1 as f64 * f64::from(resolution)) / ARCSEC_PER_DEG,
        };

        let mut file = BufReader::new(File::open(path)?);

        let samples = {
            let mut samples = Vec::with_capacity(cols * rows);

            for _ in 0..(cols * rows) {
                let sample = file.read_i16::<BE>()?;
                samples.push(sample);
            }

            assert_eq!(samples.len(), dimensions.0 * dimensions.1);
            Storage::Parsed(samples.into_boxed_slice())
        };

        let min_elev = samples.min_elev();
        let max_elev = samples.max_elev();

        Ok(Self {
            sw_corner,
            ne_corner,
            resolution,
            dimensions,
            min_elev,
            max_elev,
            samples,
        })
    }

    /// Returns Self using the memory-mapped file as storage.
    pub fn memmap<P: AsRef<Path>>(path: P) -> Result<Self, HError> {
        let (resolution, dimensions @ (cols, rows)) = extract_resolution(&path)?;
        let sw_corner = {
            let Coord { x, y } = parse_sw_corner(&path)?;
            Coord {
                x: f64::from(x),
                y: f64::from(y),
            }
        };

        let ne_corner = Coord {
            y: sw_corner.y + (cols as f64 * f64::from(resolution)) / ARCSEC_PER_DEG,
            x: sw_corner.x + (rows as f64 * f64::from(resolution)) / ARCSEC_PER_DEG,
        };

        let samples = {
            let file = File::open(path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            Storage::Mapped(mmap)
        };

        let min_elev = samples.min_elev();
        let max_elev = samples.max_elev();

        Ok(Self {
            sw_corner,
            ne_corner,
            resolution,
            dimensions,
            min_elev,
            max_elev,
            samples,
        })
    }

    #[cfg(feature = "kml")]
    pub fn to_kml(&self) -> Vec<Kml> {
        self.iter()
            .take(10)
            .map(|sample| Kml::Polygon(sample.to_kml()))
            .collect()
    }

    /// Returns the lowest elevation sample in this tile.
    pub fn min_elev(&mut self) -> i16 {
        self.min_elev
    }

    /// Returns the highest elevation sample in this tile.
    pub fn max_elev(&mut self) -> i16 {
        self.max_elev
    }

    /// Rreturns this tile's resolution in arcseconds per sample.
    pub fn resolution(&self) -> u8 {
        self.resolution
    }

    /// Returns the sample at the given geo coordinates.
    pub fn get(&self, coord: Coord) -> Option<i16> {
        let _2d_idx @ (idx_x, idx_y) = self.coord_to_xy(coord);
        if idx_x < self.dimensions.0 && idx_y < self.dimensions.1 {
            let _1d_idx = self.xy_to_linear_index(_2d_idx);
            Some(self.samples.get_unchecked(_1d_idx))
        } else {
            None
        }
    }

    /// Returns the sample at the given geo coordinates.
    pub fn get_unchecked(&self, coord: Coord) -> i16 {
        let _2d_idx = self.coord_to_xy(coord);
        let _1d_idx = self.xy_to_linear_index(_2d_idx);
        self.samples.get_unchecked(_1d_idx)
    }

    /// Returns and iterator over `self`'s grid squares.
    pub fn iter(&self) -> impl Iterator<Item = Sample<'_>> + '_ {
        (0..(self.dimensions.0 * self.dimensions.1)).map(|index| Sample {
            parent: self,
            index,
        })
    }
}

/// Private API
impl Tile {
    fn get_xy(&self, (x, y): (usize, usize)) -> i16 {
        let _1d_idx = self.xy_to_linear_index((x, y));
        self.samples.get_unchecked(_1d_idx)
    }

    fn coord_to_xy(&self, coord: Coord<f64>) -> (usize, usize) {
        let c = ARCSEC_PER_DEG / f64::from(self.resolution);
        // TODO: do we need to compensate for cell width. If so, does
        //       the following accomplish that? It seems to in the
        //       Mt. Washington test.
        let sample_center_compensation = 1. / (c * 2.);
        let cc = sample_center_compensation;
        let x = ((coord.x - self.sw_corner.x + cc) * c) as usize;
        let y = ((coord.y - self.sw_corner.y + cc) * c) as usize;
        (x, y)
    }

    fn linear_index_to_xy(&self, idx: usize) -> (usize, usize) {
        let y = idx / self.dimensions.0;
        let x = idx % self.dimensions.1;
        (x, self.dimensions.1 - 1 - y)
    }

    fn xy_to_linear_index(&self, (x, y): (usize, usize)) -> usize {
        self.dimensions.0 * (self.dimensions.1 - y - 1) + x
    }

    fn xy_to_polygon(&self, (x, y): (usize, usize)) -> Polygon<f64> {
        let center = Coord {
            x: self.sw_corner.x + (y as f64 * f64::from(self.resolution)) / ARCSEC_PER_DEG,
            y: self.sw_corner.y + (x as f64 * f64::from(self.resolution)) / ARCSEC_PER_DEG,
        };
        polygon(&center, f64::from(self.resolution))
    }
}

/// Generate a `res`-arcsecond square around `center`.
fn polygon(center: &Coord<f64>, res: f64) -> Polygon<f64> {
    let delta = res * HALF_ARCSEC;
    let n = center.y + delta;
    let e = center.x + delta;
    let s = center.y - delta;
    let w = center.x - delta;
    polygon![
        (x: w, y: s),
        (x: e, y: s),
        (x: e, y: n),
        (x: w, y: n),
        (x: w, y: s),
    ]
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
        self.parent.samples.get_unchecked(self.index)
    }

    pub fn polygon(&self) -> Polygon {
        self.parent
            .xy_to_polygon(self.parent.linear_index_to_xy(self.index))
    }

    #[cfg(feature = "kml")]
    pub fn to_kml(&self) -> KmlPolygon {
        let geo_poly = self.polygon();
        let outer_ring_coords: Vec<KmlCoord<f64>> = geo_poly
            .exterior()
            .coords()
            .map(|Coord { x, y }| KmlCoord {
                x: *x,
                y: *y,
                z: None,
            })
            .collect();

        let outer_ring = {
            let mut outer_ring = KmlLinearRing::default();
            outer_ring.coords = outer_ring_coords;
            outer_ring
        };
        KmlPolygon::new(outer_ring, Vec::new())
    }
}

fn extract_resolution<P: AsRef<Path>>(path: P) -> Result<(u8, (usize, usize)), HError> {
    const RES_1_ARCSECONDS_FIBE_BEN: u64 = 3601 * 3601 * size_of::<u16>() as u64;
    const RES_3_ARCSECONDS_FIBE_BEN: u64 = 1201 * 1201 * size_of::<u16>() as u64;
    match path.as_ref().metadata().map(|m| m.len())? {
        RES_1_ARCSECONDS_FIBE_BEN => Ok((1, (3601, 3601))),
        RES_3_ARCSECONDS_FIBE_BEN => Ok((3, (1201, 1201))),
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
mod _1_arc_second {
    use super::{
        extract_resolution, parse_sw_corner, BufReader, Coord, File, ReadBytesExt, Tile, BE,
    };
    use std::path::PathBuf;

    fn one_arcsecond_dir() -> PathBuf {
        [
            env!("CARGO_MANIFEST_DIR"),
            "..",
            "data",
            "nasadem",
            "1arcsecond",
        ]
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
        Tile::parse(path).unwrap();
    }

    #[test]
    fn test_tile_index() {
        let mut path = one_arcsecond_dir();
        path.push("N44W072.hgt");
        let raw_file_samples = {
            let mut file_data = Vec::new();
            let mut file = BufReader::new(File::open(&path).unwrap());
            while let Ok(sample) = file.read_i16::<BE>() {
                file_data.push(sample);
            }
            file_data
        };
        let parsed_tile = Tile::parse(&path).unwrap();
        let mapped_tile = Tile::memmap(&path).unwrap();
        let mut idx = 0;
        for row in (0..3601).rev() {
            for col in 0..3601 {
                assert_eq!(raw_file_samples[idx], parsed_tile.get_xy((col, row)));
                assert_eq!(raw_file_samples[idx], mapped_tile.get_xy((col, row)));
                idx += 1;
            }
        }
    }

    #[test]
    fn test_tile_geo_index() {
        let mut path = one_arcsecond_dir();
        path.push("N44W072.hgt");
        let mut tile = Tile::parse(&path).unwrap();
        let mt_washington = Coord {
            y: 44.2705,
            x: -71.30325,
        };
        assert_eq!(tile.get_unchecked(mt_washington), tile.max_elev());
    }

    #[test]
    fn test_tile_index_conversions() {
        let mut path = one_arcsecond_dir();
        path.push("N44W072.hgt");
        let tile = Tile::parse(&path).unwrap();
        for row in (0..3601).rev() {
            for col in 0..3601 {
                let _1d = tile.xy_to_linear_index((col, row));
                let roundtrip_2d = tile.linear_index_to_xy(_1d);
                assert_eq!((col, row), roundtrip_2d);
            }
        }
    }
}

#[cfg(test)]
mod _3_arc_second {
    use super::{
        extract_resolution, parse_sw_corner, BufReader, Coord, File, Polygon, ReadBytesExt, Tile,
        BE,
    };
    use geo_types::LineString;
    use std::path::PathBuf;

    #[cfg(feature = "kml")]
    use kml::types::{Kml, KmlDocument, KmlVersion};

    fn three_arcsecond_dir() -> PathBuf {
        [
            env!("CARGO_MANIFEST_DIR"),
            "..",
            "data",
            "nasadem",
            "3arcsecond",
        ]
        .iter()
        .collect()
    }

    #[test]
    fn test_parse_hgt_name() {
        let mut path = three_arcsecond_dir();
        path.push("N44W072.hgt");
        let sw_corner = parse_sw_corner(&path).unwrap();
        let resolution = extract_resolution(&path).unwrap();
        assert_eq!(sw_corner, Coord { x: -72, y: 44 });
        assert_eq!(resolution, (3, (1201, 1201)));
    }

    #[test]
    fn test_tile_open() {
        let mut path = three_arcsecond_dir();
        path.push("N44W072.hgt");
        Tile::parse(path).unwrap();
    }

    #[test]
    fn test_tile_index() {
        let mut path = three_arcsecond_dir();
        path.push("N44W072.hgt");
        let tile = Tile::parse(&path).unwrap();
        let raw_file_samples = {
            let mut file_data = Vec::new();
            let mut file = BufReader::new(File::open(path).unwrap());
            while let Ok(sample) = file.read_i16::<BE>() {
                file_data.push(sample);
            }
            file_data
        };
        let mut idx = 0;
        for row in (0..1201).rev() {
            for col in 0..1201 {
                assert_eq!(raw_file_samples[idx], tile.get_xy((col, row)));
                idx += 1;
            }
        }
    }

    // #[test]
    // fn test_tile_geo_index() {
    //     let mut path = three_arcsecond_dir();
    //     path.push("N44W072.hgt");
    //     let tile = Tile::parse(&path).unwrap();
    //     let mt_washington = Coord {
    //         y: 44.2705,
    //         x: -71.30325,
    //     };
    //     // TODO: is there an error in indexing or is the 3 arc-second
    //     //       dataset smeared?
    //     assert_eq!(tile.get_coord(mt_washington), tile.max_elev());
    // }

    #[test]
    fn test_tile_index_conversions() {
        let mut path = three_arcsecond_dir();
        path.push("N44W072.hgt");
        let parsed_tile = Tile::parse(&path).unwrap();
        for row in (0..1201).rev() {
            for col in 0..1201 {
                let _1d = parsed_tile.xy_to_linear_index((col, row));
                let roundtrip_2d = parsed_tile.linear_index_to_xy(_1d);
                assert_eq!((col, row), roundtrip_2d);
            }
        }
    }

    #[test]
    fn test_xy_to_polygon() {
        let mut path = three_arcsecond_dir();
        path.push("N44W072.hgt");
        let parsed_tile = Tile::parse(&path).unwrap();
        assert_eq!(
            parsed_tile.xy_to_polygon((0, 0)),
            Polygon::new(
                LineString::from(vec![
                    (-72.000_416_666_666_67, 43.999_583_333_333_334),
                    (-71.999_583_333_333_33, 43.999_583_333_333_334),
                    (-71.999_583_333_333_33, 44.000_416_666_666_666),
                    (-72.000_416_666_666_67, 44.000_416_666_666_666),
                    (-72.000_416_666_666_67, 43.999_583_333_333_334),
                ]),
                vec![],
            )
        );
    }

    #[cfg(feature = "kml")]
    #[test]
    fn test_to_kml() {
        let kml_doc = {
            let mut path = three_arcsecond_dir();
            path.push("N44W072.hgt");
            let parsed_tile = Tile::parse(&path).unwrap();
            let elements = parsed_tile.to_kml();
            Kml::KmlDocument(KmlDocument {
                version: KmlVersion::V22,
                elements,
                ..Default::default()
            })
        };
        let out = std::io::BufWriter::new(File::create("/tmp/N44W072.kml").unwrap());
        let mut writer = kml::KmlWriter::from_writer(out);
        writer.write(&kml_doc).unwrap();
    }
}
