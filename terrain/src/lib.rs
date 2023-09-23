mod constants;
mod error;
mod haversine;
mod math;
mod profile;
mod tiles;

pub use crate::{
    error::TerrainError,
    profile::Profile,
    tiles::{TileMode, Tiles},
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
