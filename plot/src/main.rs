mod options;

use anyhow::Error as AnyError;
use clap::Parser;
use options::{Cli, Command as CliCmd};
use serde::Serialize;
use std::{io::Write, path::Path};
use terrain::{Profile, TileMode, Tiles};
use textplots::{Chart, Plot, Shape};

fn main() -> Result<(), AnyError> {
    let Cli {
        srtm_dir,
        step_size,
        start,
        dest,
        cmd,
    } = Cli::parse();

    env_logger::init();

    let tile_src = Tiles::new(srtm_dir, TileMode::MemMap)?;
    let terrain_profile = Profile::new(start.0, step_size, dest.0, &tile_src)?;

    match cmd {
        CliCmd::Csv => print_csv(terrain_profile),
        CliCmd::Plot { out: Some(out) } => plot_svg(terrain_profile, &out),
        CliCmd::Plot { out: None } => plot_ascii(terrain_profile),
        CliCmd::Json => json(terrain_profile),
    }
}

/// # Example with gnuplot
///
/// ```sh
/// cargo run --release -- --srtm-dir=data/nasadem/3arcsecond/ -z90 --start=44.28309806603165,-71.30830716441369 --dest=44.25628098424278,-71.2972073283768 csv | tr ',' ' ' | gnuplot -p -e "plot '-' using 1:4 with line"
/// ```
fn print_csv(profile: Profile<f64>) -> Result<(), AnyError> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "Distance,Longitude,Latitude,Elevation")?;
    for (i, (elevation, point)) in profile
        .terrain
        .iter()
        .zip(profile.great_circle.iter())
        .enumerate()
    {
        let distance = (profile.distance * i as f64) as usize / (profile.terrain.len() - 1);
        let longitude = point.x();
        let latitude = point.y();
        writeln!(stdout, "{distance},{longitude},{latitude},{elevation}",)?;
    }
    Ok(())
}

fn plot_svg(_profile: Profile<f64>, _out: &Path) -> Result<(), AnyError> {
    unimplemented!()
}

fn plot_ascii(profile: Profile<f64>) -> Result<(), AnyError> {
    let plot_data: Vec<(f32, f32)> = profile
        .terrain
        .iter()
        .enumerate()
        .map(|(idx, elev)| (f32::from(idx as u16), f32::from(*elev)))
        .collect();
    Chart::new(400, 150, 0.0, plot_data.len() as f32)
        .lineplot(&Shape::Lines(&plot_data))
        .display();
    Ok(())
}

fn json(profile: Profile<f64>) -> Result<(), AnyError> {
    #[derive(Serialize)]
    struct JsonEntry {
        location: [f64; 2],
        elevation: i16,
    }

    let reshaped: Vec<JsonEntry> = profile
        .great_circle
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
