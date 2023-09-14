//! NASADEM file aggregator.

use crate::TerrainError;
use dashmap::{mapref::entry::Entry, DashMap};
use geo_types::Coord;
use nasadem::{HError, Tile};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct TileSource {
    /// Directory containing NASADEM HGT tile files.
    tile_dir: PathBuf,
    /// Tiles which have been loaded on demand.
    tiles: DashMap<Coord<i32>, Arc<Tile>>,
}

impl TileSource {
    pub fn new(tile_dir: PathBuf) -> Result<Self, TerrainError> {
        let mut has_height_files = false;

        for entry in std::fs::read_dir(&tile_dir)? {
            let path = entry?.path();
            if Some("hgt") == path.extension().and_then(|ext| ext.to_str()) {
                has_height_files = true;
                break;
            }
        }

        if has_height_files {
            let tiles = DashMap::new();
            Ok(Self { tile_dir, tiles })
        } else {
            Err(TerrainError::Path(tile_dir))
        }
    }

    /// Returns the tile containiong `coord`, if any.
    ///
    /// This TileSource will attempt to load the tile from disk if it
    /// doesn't already have it in memory.
    pub fn get(&self, coord: Coord<f64>) -> Result<Option<Arc<Tile>>, TerrainError> {
        let sw_corner = sw_corner(coord);

        if let Entry::Vacant(e) = self.tiles.entry(sw_corner) {
            let tile = {
                let file_name = file_name(sw_corner);
                let tile_path: PathBuf = [&self.tile_dir, Path::new(&file_name)].iter().collect();
                let tile = match Tile::memmap(tile_path) {
                    Ok(tile) => tile,
                    Err(HError::Io(e)) if e.kind() == ErrorKind::NotFound => return Ok(None),
                    err => err?,
                };
                Arc::new(tile)
            };
            e.insert(tile);
        }

        Ok(self.tiles.get(&sw_corner).as_deref().cloned())
    }
}

/// Returns the southwest corner as integers for coord.
fn sw_corner(Coord { x, y }: Coord<f64>) -> Coord<i32> {
    Coord {
        x: (x.floor() as i32),
        y: (y.floor() as i32),
    }
}

/// Returns the expected file name for coord
fn file_name(Coord { x, y }: Coord<i32>) -> String {
    let (n_s, lat) = {
        let lat = y.abs();
        let n_s = match y.is_positive() {
            true => 'N',
            false => 'S',
        };
        (n_s, lat)
    };
    let (e_w, lon) = {
        let lon = x.abs();
        let e_w = match x.is_positive() {
            true => 'E',
            false => 'W',
        };
        (e_w, lon)
    };
    format!("{n_s}{lat:02}{e_w}{lon:03}.hgt")
}

#[cfg(test)]
mod tests {
    use super::{file_name, sw_corner, Coord, PathBuf, TileSource};

    const MT_WASHINGTON: Coord = Coord {
        y: 44.2705,
        x: -71.30325,
    };

    const SOUTH_POLE: Coord = Coord { y: -90.0, x: 0.0 };

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
    fn test_get_invalid() {
        let tile_src = TileSource::new(three_arcsecond_dir()).unwrap();
        assert!(tile_src.get(SOUTH_POLE).unwrap().is_none());
    }

    #[test]
    fn test_get() {
        let tile_src = TileSource::new(three_arcsecond_dir()).unwrap();
        let tile = tile_src.get(MT_WASHINGTON).unwrap().unwrap();
        assert_eq!(tile.get_unchecked(MT_WASHINGTON), 1903);
    }

    #[test]
    fn test_file_name() {
        let sw_corner = sw_corner(MT_WASHINGTON);
        let actual = file_name(sw_corner);
        assert_eq!(actual, "N44W072.hgt");
    }
}
