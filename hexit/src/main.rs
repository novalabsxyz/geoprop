mod combine;
mod options;
mod progress_bar;
mod tesselate;

use anyhow::Result;
use clap::Parser;
use options::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli {
        Cli::Tessellate(tesselate) => tesselate.run(),
        Cli::Combine(combine) => combine.run(),
    }
}
