//! NASADEM file aggregator.

use crate::TerrainError;
use dashmap::DashMap;
use geo::geometry::Coord;
use log::debug;
use nasadem::{NasademError, Tile};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Floating point type used for tile lookup.
pub type C = f64;

#[derive(Clone)]
pub struct Tiles {
    /// Directory containing NASADEM HGT tile files.
    tile_dir: PathBuf,

    /// How to load tiles (in-memory or mapped).
    tile_mode: TileMode,

    /// Tiles which have been loaded on demand.
    tiles: DashMap<Coord<i16>, Arc<Tile>>,
}

impl Tiles {
    pub fn new(tile_dir: PathBuf, tile_mode: TileMode) -> Result<Self, TerrainError> {
        let mut has_height_files = false;

        // Let's try to fail early be checking that tile_dir has at
        // least one `hgt` file.
        for entry in std::fs::read_dir(&tile_dir)? {
            let path = entry?.path();
            if Some("hgt") == path.extension().and_then(std::ffi::OsStr::to_str) {
                has_height_files = true;
                break;
            }
        }

        if has_height_files {
            let tiles = DashMap::new();
            Ok(Self {
                tile_dir,
                tile_mode,
                tiles,
            })
        } else {
            Err(TerrainError::Path(tile_dir))
        }
    }

    /// Returns the tile containiong `coord`, if any.
    ///
    /// `Tiles` will attempt to fetch the tile from disk if it doesn't
    /// already have it in memory.
    pub fn get(&self, coord: Coord<C>) -> Result<Arc<Tile>, TerrainError> {
        let sw_corner = sw_corner(coord);
        self.tiles
            .entry(sw_corner)
            .or_try_insert_with(|| match self.load_tile(sw_corner) {
                Ok(tile) => Ok(Arc::new(tile)),
                Err(TerrainError::Nasadem(NasademError::Io(e)))
                    if e.kind() == ErrorKind::NotFound =>
                {
                    Ok(Arc::new(Self::load_tombstone(sw_corner)))
                }
                Err(e) => Err(e),
            })
            .map(|r| r.clone())
    }
}

/// Private API.
impl Tiles {
    fn load_tile(&self, sw_corner: Coord<i16>) -> Result<Tile, TerrainError> {
        let tile_path = {
            let file_name = file_name(sw_corner);
            let mut tile_path: PathBuf = [&self.tile_dir, Path::new(&file_name)].iter().collect();
            if !tile_path.exists() {
                let file_name = file_name.to_lowercase();
                tile_path = [&self.tile_dir, Path::new(&file_name)].iter().collect();
            }
            tile_path
        };
        debug!("loading {tile_path:?}");
        match self.tile_mode {
            TileMode::InMem => Ok(Tile::load(tile_path)?),
            TileMode::MemMap => Ok(Tile::memmap(tile_path)?),
        }
    }

    fn load_tombstone(sw_corner: Coord<i16>) -> Tile {
        debug!("loading tombstone in lieu of missing tile for {sw_corner:?}");
        Tile::tombstone(sw_corner)
    }
}

/// How to handle tile.
///
/// The trade off between loading tile data into memory versus memory
/// mapping is not obvious, and you should measure both before
/// deciding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileMode {
    /// Parse tile and load into memory.
    ///
    /// Note that this can consume gigabytes of RAM when loading many
    /// tiles.
    InMem,

    /// Memory map file contents.
    MemMap,
}

/// Returns the southwest corner as integers for coord.
fn sw_corner(Coord { x, y }: Coord<C>) -> Coord<i16> {
    #[allow(clippy::cast_possible_truncation)]
    Coord {
        x: (x.floor() as i16),
        y: (y.floor() as i16),
    }
}

/// Returns the expected file name for coord
fn file_name(Coord { x, y }: Coord<i16>) -> String {
    let (n_s, lat) = {
        let lat = y.abs();
        let n_s = if y.is_negative() { 'S' } else { 'N' };
        (n_s, lat)
    };
    let (e_w, lon) = {
        let lon = x.abs();
        let e_w = if x.is_negative() { 'W' } else { 'E' };
        (e_w, lon)
    };
    format!("{n_s}{lat:02}{e_w}{lon:03}.hgt")
}

#[cfg(test)]
mod tests {
    use super::{file_name, sw_corner, Coord, TileMode, Tiles};

    const MT_WASHINGTON: Coord = Coord {
        y: 44.2705,
        x: -71.30325,
    };

    const SOUTH_POLE: Coord = Coord { y: -90.0, x: 0.0 };

    #[test]
    fn test_missing_tile_returns_0() {
        let tile_src = Tiles::new(crate::three_arcsecond_dir(), TileMode::MemMap).unwrap();
        let tile = tile_src.get(SOUTH_POLE).unwrap();
        let elevation = tile.get(SOUTH_POLE).unwrap();
        assert_eq!(elevation, 0);
    }

    #[test]
    fn test_get() {
        let tile_src = Tiles::new(crate::three_arcsecond_dir(), TileMode::MemMap).unwrap();
        let tile = tile_src.get(MT_WASHINGTON).unwrap();
        assert_eq!(tile.get_unchecked(MT_WASHINGTON), 1903);
    }

    #[test]
    fn test_file_name() {
        let name = file_name(sw_corner(Coord {
            y: 0.0 + f64::EPSILON,
            x: 0.0 + f64::EPSILON,
        }));
        assert_eq!(name, "N00E000.hgt");

        let name = file_name(sw_corner(Coord {
            y: 0.0 + f64::EPSILON,
            x: 0.0 - f64::EPSILON,
        }));
        assert_eq!(name, "N00W001.hgt");

        let name = file_name(sw_corner(Coord {
            y: 0.0 - f64::EPSILON,
            x: 0.0 - f64::EPSILON,
        }));
        assert_eq!(name, "S01W001.hgt");

        let name = file_name(sw_corner(Coord {
            y: 0.0 - f64::EPSILON,
            x: 0.0 + f64::EPSILON,
        }));
        assert_eq!(name, "S01E000.hgt");
    }
}
