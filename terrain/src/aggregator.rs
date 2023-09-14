//! NASADEM file aggregator.

use geo_types::Coord;
use nasadem::Tile;
use std::{
    collections::HashMap,
    path::PathBuf,
    // sync::{Arc, RwLock},
};

pub struct TileSource {
    /// Directory containing NASADEM HGT tile files.
    _tile_dir: PathBuf,
    /// Tiles which have been loaded on demand.
    _tiles: HashMap<Coord<i32>, Tile>,
}

impl TileSource {
    pub fn new(tile_dir: PathBuf) -> Self {
        let tiles = HashMap::new();
        Self {
            _tile_dir: tile_dir,
            _tiles: tiles,
        }
    }

    /// Returns the tile containiong `coord`, if any.
    ///
    /// This TileSource will attempt to load the tile from disk if it
    /// doesn't already have it in memory.
    pub fn get(&self, _coord: Coord<f64>) -> Option<&Tile> {
        unimplemented!()
    }
}
