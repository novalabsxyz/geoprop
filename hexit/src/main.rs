mod combine;
mod lookup;
mod mask;
mod options;
mod progress;
mod tesselate;

use anyhow::Result;
use clap::Parser;
use options::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli {
        Cli::Tessellate(tesselate) => tesselate.run(),
        Cli::Combine(combine) => combine.run(),
        Cli::Lookup(lookup) => lookup.run(),
    }
}
