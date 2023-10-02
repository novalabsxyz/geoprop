mod constants;
mod error;
mod math;
mod profile;
mod tiles;

pub use crate::{
    error::TerrainError,
    profile::Profile,
    tiles::{TileMode, Tiles},
};

pub use geo;

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
