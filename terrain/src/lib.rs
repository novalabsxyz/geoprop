//! # Terrain Profiling
//!
//! The `terrain` module facilitates the aggregation and sourcing of
//! NASADEM tiles, allowing the generation of 1D elevation profiles
//! between any two points on Earth.
//!
//! ## Motivating Example
//!
//! Suppose we want to determine the terrain obstruction a ray will
//! encounter when following the path on the map from east to west:
//!
//! <iframe src="https://www.google.com/maps/d/u/0/embed?mid=1Q4TbMv-ZmAa4Uf6FizvkhQD3Ww2A498&ehbc=2E312F" width="100%" height="480"></iframe>
//!
//! **Example Code**
//!
//! ```rust
//! # use terrain::{TerrainError, Profile, Tiles, TileMode, geo::coord};
//! #
//! # fn main() -> Result<(), TerrainError> {
//! #     let srtm_dir: std::path::PathBuf =
//! #         [
//! #             env!("CARGO_MANIFEST_DIR"),
//! #             "..",
//! #             "data",
//! #             "nasadem",
//! #             "3arcsecond",
//! #         ]
//! #         .iter()
//! #         .collect();
//! #
//! // The `Tiles` struct handles the loading of SRTM tiles from disk.
//! // In this example, `srtm_dir` is a flat directory containing
//! // 3-arcsecond SRTM files.
//! let tiles = Tiles::new(srtm_dir, TileMode::MemMap)?;
//!
//! // Define start and end coordinates, where x is longitude and y
//! // latitude.
//! let start = coord!(x: -119.8716916239494, y: 39.15632968072683);
//! let end = coord!(x: -120.2510792587534, y: 38.99292143188696);
//!
//! // Since we have 3-arcsecond tiles, let's request a maximum
//! // distance of 3-arcseconds (90 meters) between each elevation
//! // sample.
//! let max_step_m = 90.0;
//!
//! // Our ray starts 2 meters above ground and aims for 3 meters
//! // above ground at the destination.
//! let start_alt_m = 2.0;
//! let end_alt_m = 3.0;
//!
//! // Build a terrain profile with specified parameters.
//! let profile = Profile::builder()
//!     .start(start)
//!     .start_alt(start_alt_m)
//!     .max_step(max_step_m)
//!     .end(end)
//!     .end_alt(end_alt_m)
//!     .earth_curve(true)
//!     .normalize(true)
//!     .build(&tiles)?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! **Output Data**
//!
//! Here's an externally generated plot (plotting not included in this
//! crate) of `profile`'s [`los_elev_m`] and [`terrain_elev_m`] over its
//! [`distances_m`]:
//!
//! ![Lake Tahoe](https://github.com/JayKickliter/geoprof/assets/2551201/b8c94b4b-017c-4dd1-8a87-37c808ccea2b)
//!
//! [`los_elev_m`]: struct.Profile.html#structfield.los_elev_m
//! [`terrain_elev_m`]: struct.Profile.html#structfield.terrain_elev_m
//! [`distances_m`]: struct.Profile.html#structfield.distances_m

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
