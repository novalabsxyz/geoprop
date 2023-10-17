//! # Radio Frequency Propogation
//!
//! `propah` provides radio propogation modeling routines.

mod error;
pub mod fresnel;
mod math;
pub mod p2p;

pub use {
    crate::{error::PropahError, p2p::Point2Point},
    geo, terrain,
};

/// Speed of light in m/s
const C: usize = 299_792_458;
