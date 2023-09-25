use anyhow::{anyhow, Error as AnyError};
use clap::{Parser, Subcommand};
use geo::geometry::Coord;
use std::{path::PathBuf, str::FromStr};

/// A tool for enerating terrain profiles.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Directory containing SRTM hgt tiles.
    #[arg(short, long)]
    pub srtm_dir: PathBuf,

    /// Path incremental step size.
    #[arg(short = 'z', long, default_value_t = 90.0)]
    pub step_size: f64,

    /// Start "lat,lon,alt"
    #[arg(long)]
    pub start: LatLonAlt,

    /// Destination "lat,lon"
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

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print terrain values to stdout.
    Csv,

    /// Print terrain values to stdout.
    Json,

    /// Export an SVG.
    Plot {
        /// SVG file path.
        out: Option<PathBuf>,
    },
}
