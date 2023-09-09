mod error;
mod input;
mod options;

use clap::Parser;
use error::HError;

fn main() -> Result<(), HError> {
    let _cli = options::Cli::parse();
    Ok(())
}
