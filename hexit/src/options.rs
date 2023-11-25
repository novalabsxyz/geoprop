use clap::{Args, Parser};
use h3o::Resolution;
use std::path::PathBuf;

/// Generate H3 tessellated polyfills from raster data.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    Tessellate(Tesselate),

    Combine(Combine),

    Lookup(Lookup),

    Json(Json),
}

/// Generate a tessellated list of (cell, elevation) for each
/// input file.
#[derive(Debug, Clone, Args)]
pub struct Tesselate {
    /// Path GeoJSON mask.
    ///
    /// Any tiles which do not intersect the mask are ignored.
    #[arg(short, long)]
    pub mask: Option<PathBuf>,

    /// Reprocess height file even if corresponding output already
    /// exists.
    #[arg(short = 'O', long)]
    pub overwrite: bool,

    /// Amount of compression.
    #[arg(short, long, default_value_t = 6)]
    pub compression: u32,

    #[arg(short, long, default_value_t = Resolution::Twelve)]
    pub resolution: Resolution,

    /// Output directory.
    #[arg(short, long)]
    pub out_dir: PathBuf,

    /// Input SRTM elevation (.hgt) files.
    pub input: Vec<PathBuf>,
}

/// Combine previously tesselated files into a single
#[derive(Debug, Clone, Args)]
pub struct Combine {
    #[arg(short, long)]
    pub tolerance: i16,

    #[arg(short, long)]
    pub out: PathBuf,

    /// Input tessaltions.
    pub input: Vec<PathBuf>,
}

/// Lookup value for H3 cell in a disktree.
#[derive(Debug, Clone, Args)]
pub struct Lookup {
    /// Iterate through the disktree instead of `get`ting the value.
    #[arg(short, long)]
    pub iter: bool,
    pub disktree: PathBuf,
    pub cell: String,
}

/// Output kepler.gl compatible JSON within the given mask.
#[derive(Debug, Clone, Args)]
pub struct Json {
    /// Source resolution.
    #[arg(short, long)]
    pub resolution: Resolution,

    /// Path GeoJSON mask.
    ///
    /// Any samples which do not intersect the mask are ignored.
    pub mask: PathBuf,

    pub disktree: PathBuf,
}
