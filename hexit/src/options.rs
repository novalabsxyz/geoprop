use clap::{Args, Parser};
use h3o::Resolution;
use std::path::PathBuf;

/// Generate H3 tessellated polyfills from raster data.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    /// Generate a tessellated list of (cell, elevation) for each
    /// input file.
    Tessellate(Tesselate),

    /// Combine previously tesselated files into a single
    Combine(Combine),
}

#[derive(Debug, Clone, Args)]
pub struct Tesselate {
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

#[derive(Debug, Clone, Args)]
pub struct Combine {
    #[arg(short, long, default_value_t = Resolution::Ten)]
    pub resolution: Resolution,

    #[arg(short, long)]
    pub out: PathBuf,

    /// Input tessaltions.
    pub input: Vec<PathBuf>,
}
