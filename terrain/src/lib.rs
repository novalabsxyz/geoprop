mod error;
mod profile;
mod tile_source;

pub use crate::{
    error::TerrainError,
    profile::Profile,
    tile_source::{TileMode, TileSource},
};

#[cfg(test)]
fn three_arcsecond_dir() -> std::path::PathBuf {
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
