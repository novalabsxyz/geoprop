mod options;

use anyhow::Error as AnyError;
use clap::Parser;
use options::{Cli, Command as CliCmd};
use serde::Serialize;
use std::io::Write;
use terrain::{Profile, TileMode, TileSource};

const _90m: f64 = 90.0;

fn main() -> Result<(), AnyError> {
    let Cli {
        srtm_dir,
        start,
        dest,
        cmd,
    } = Cli::parse();

    let tile_src = TileSource::new(srtm_dir, TileMode::MemMap)?;
    let terrain_profile = Profile::new(start.0, _90m, dest.0, &tile_src)?;

    match cmd {
        CliCmd::Display => display(terrain_profile),
        CliCmd::Plot { out: _out } => unimplemented!(),
        CliCmd::Json => json(terrain_profile),
    }
}

fn display(profile: Profile<f64>) -> Result<(), AnyError> {
    let mut stdout = std::io::stdout().lock();
    for (i, elevation) in profile.terrain.into_iter().enumerate() {
        writeln!(stdout, "{i:4}: {elevation}");
    }
    Ok(())
}

fn plot(profile: Profile<f64>) -> Result<(), AnyError> {
    unimplemented!()
}


fn json(profile: Profile<f64>) -> Result<(), AnyError> {
    #[derive(Serialize)]
    struct JsonEntry {
        location: [f64; 2],
        elevation: i16,
    }

    let reshaped: Vec<JsonEntry> = profile
        .points
        .iter()
        .zip(profile.terrain.iter())
        .map(|(point, elev)| JsonEntry {
            location: [point.x(), point.y()],
            elevation: *elev,
        })
        .collect();
    let json = serde_json::to_string(&reshaped)?;
    println!("{}", json);
    Ok(())
}
