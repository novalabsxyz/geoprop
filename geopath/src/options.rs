use anyhow::{anyhow, Error as AnyError};
use clap::{Parser, Subcommand};
use geo::geometry::Coord;
use std::{path::PathBuf, str::FromStr};

/// Generate point-to-point terrain profiles.
#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Directory containing SRTM hgt tiles.
    #[arg(short, long)]
    pub srtm_dir: PathBuf,

    #[arg(long, default_value_t = false)]
    pub rfprop: bool,

    /// Maximum path incremental step size, in meters.
    #[arg(short, long, default_value_t = 90.0)]
    pub max_step: f64,

    /// Add earth curvature to terrain values.
    #[arg(short, long, default_value_t = false)]
    pub earth_curve: bool,

    /// Center earth curve so that midpoint between start and end is
    /// the highest.
    #[arg(short, long, default_value_t = false)]
    pub normalize: bool,

    /// Start "lat,lon,alt", where 'alt' is meters above ground.
    #[arg(long)]
    pub start: LatLonAlt,

    /// Destination "lat,lon,alt", where 'alt' is meters above ground.
    #[arg(long)]
    pub dest: LatLonAlt,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Clone, Debug)]
pub struct LatLonAlt(pub Coord<f64>, pub i16);

impl FromStr for LatLonAlt {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, AnyError> {
        let (lat_str, lon_str, alt_str) = {
            let idx = s.find(',').ok_or(anyhow!("not a valid lat,lon,alt"))?;
            let (lat_str, lon_alt_str) = s.split_at(idx);
            let idx = lon_alt_str[1..]
                .find(',')
                .ok_or(anyhow!("not a valid lat,lon,alt"))?;
            let (lon_str, alt_str) = lon_alt_str[1..].split_at(idx);
            (lat_str, lon_str, &alt_str[1..])
        };
        let lat = f64::from_str(lat_str)?;
        let lon = f64::from_str(lon_str)?;
        let alt = i16::from_str(alt_str)?;
        Ok(Self(Coord { y: lat, x: lon }, alt))
    }
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Print terrain values to stdout.
    Csv,

    /// Print terrain values to stdout.
    Json,

    /// Plot to terminal.
    Plot,
}
