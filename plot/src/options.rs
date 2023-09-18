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

    /// Start "lat,lon"
    #[arg(long)]
    pub start: LatLon,

    /// Destination "lat,lon"
    #[arg(long)]
    pub dest: LatLon,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Clone, Debug)]
pub struct LatLon(pub Coord<f64>);

impl FromStr for LatLon {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, AnyError> {
        let idx = s.find(",").ok_or(anyhow!("not a valid lat,lon pair"))?;
        let (lat_str, lon_str) = {
            let (lat_str, lon_str) = s.split_at(idx);
            (lat_str, &lon_str[1..])
        };
        let lat = f64::from_str(lat_str)?;
        let lon = f64::from_str(lon_str)?;
        Ok(Self(Coord { y: lat, x: lon }))
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print terrain values to screen.
    Display,

    /// Print terrain values to screen.
    Json,

    /// Export an SVG.
    Plot {
        /// SVG file path.
        out: PathBuf,
    },
}
