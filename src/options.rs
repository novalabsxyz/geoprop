use clap::{Parser, Subcommand};
use h3o::Resolution;
use std::path::PathBuf;

/// A multitool for converting geo types to hextrees.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Base
    #[arg(long)]
    res: Resolution,

    /// Compact H3 cells.
    #[arg(long)]
    compact: bool,

    /// Output directory.
    #[arg(short, long)]
    out: PathBuf,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Bsdf,
    Sdf,
    Hgt,
}
