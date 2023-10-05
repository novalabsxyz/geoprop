//! <!--Note to self: format with `cargo +nightly fmt -- --config format_code_in_doc_comments=true`-->
//!
//! # Terrain profiling
//!
//! `terrain` aggregates and sources NASADEM tiles, and generates 1d
//! elevation profiles between any two points on earth.
//!
//! ## Motivating example
//!
//! We want to know the terrain obstruction a ray will encounter if it
//! followed the line on this map from east to west:
//!
//!
//! <iframe src="https://www.google.com/maps/d/u/0/embed?mid=1Q4TbMv-ZmAa4Uf6FizvkhQD3Ww2A498&ehbc=2E312F" width="640" height="480"></iframe>
//!
//! **Code**
//!
//! ```rust
//! # use terrain::TerrainError;
//!
//! # fn main() -> Result<(), TerrainError> {
//! # let srtm_dir: std::path::PathBuf =
//! #   [
//! #       env!("CARGO_MANIFEST_DIR"),
//! #       "..",
//! #       "data",
//! #       "nasadem",
//! #       "3arcsecond",
//! #   ]
//! #   .iter()
//! #   .collect();
//! use terrain::{Profile, Tiles, TileMode, geo::coord};
//!
//! // The Tiles struct handles loading of SRTM tiles from disk.
//! // In this example, `srtm_dir` is a flat directory containing
//! // 3-arcsecond SRTM files.
//! let tiles = Tiles::new(srtm_dir, TileMode::MemMap)?;
//!
//! let start = coord!(x: -119.8716916239494, y: 39.15632968072683);
//! let end = coord!(x: -120.2510792587534, y: 38.99292143188696);
//!
//! // We know we only have 3-arcsecond tiles, so we'll request a
//! // *maximum* distance of 90 meters between each elevation sample.
//! // x = longitude, y = latitude
//! let max_step_m = 90.0;
//!
//! // Our ray is starting 2 meters above ground and aiming at 3
//! // meters above ground at the destination.
//! let start_alt_m = 2.0;
//! let end_alt_m = 3.0;
//!
//! let profile = Profile::builder()
//!                       .start(start)
//!                       .start_alt(start_alt_m)
//!                       .max_step(max_step_m)
//!                       .end(end)
//!                       .end_alt(end_alt_m)
//!                       .build(&tiles)?;
//! # Ok(())
//! # }
//! ```
//!
//! A plot (not included) `profile`'s `los_elev_m` and
//! `terrain_elev_m` over its `distances_m` looks like so:
//!
//! ![Lake Tahoe terrain profile](../../assets/lake-tahoe.svg)

mod constants;
mod error;
mod math;
mod profile;
mod tiles;

pub use crate::{
    error::TerrainError,
    profile::{Profile, ProfileBuilder},
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
