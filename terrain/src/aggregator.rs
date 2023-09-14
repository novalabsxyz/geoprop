//! NASADEM file aggregator.

use geo_types::Coord;
use nasadem::Tile;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    // sync::{Arc, RwLock},
};

pub struct TileSource {
    /// Directory containing NASADEM HGT tile files.
    tile_dir: PathBuf,
    /// Tiles which have been loaded on demand.
    tiles: HashMap<Coord<i32>, Tile>,
}

impl TileSource {
    pub fn new(tile_dir: PathBuf) -> Self {
        let tiles = HashMap::new();
        Self { tile_dir, tiles }
    }

    /// Returns the tile containiong `coord`, if any.
    ///
    /// This TileSource will attempt to load the tile from disk if it
    /// doesn't already have it in memory.
    pub fn get(&mut self, coord: Coord<f64>) -> Option<&Tile> {
        let sw_corner = sw_corner(coord);

        if let std::collections::hash_map::Entry::Vacant(e) = self.tiles.entry(sw_corner) {
            let tile = {
                let file_name = file_name(sw_corner);
                let tile_path: PathBuf = [&self.tile_dir, Path::new(&file_name)].iter().collect();
                Tile::memmap(tile_path).unwrap()
            };
            e.insert(tile);
        }

        self.tiles.get(&sw_corner)
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
    fn test_get() {
        let mut tile_src = TileSource::new(one_arcsecond_dir());
        let tile = tile_src.get(MT_WASHINGTON).unwrap();
        assert_eq!(tile.get_unchecked(MT_WASHINGTON), 1914);
    }

    #[test]
    fn test_file_name() {
        let sw_corner = sw_corner(MT_WASHINGTON);
        let actual = file_name(sw_corner);
        assert_eq!(actual, "N44W072.hgt");
    }
}
