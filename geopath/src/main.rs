mod options;

use anyhow::Error as AnyError;
use clap::Parser;
use options::{Cli, Command as CliCmd};
use serde::Serialize;
use std::{io::Write, path::Path};
use terrain::{Profile, TileMode, Tiles};
use textplots::{Chart, Plot, Shape};

fn main() -> Result<(), AnyError> {
    let cli = Cli::parse();
    let Cli {
        srtm_dir,
        rfprop,
        max_step: step_size,
        earth_curve,
        normalize,
        start,
        dest,
        cmd,
    } = &cli;

    env_logger::init();

    let tile_src = Tiles::new(srtm_dir.clone(), TileMode::MemMap)?;
    let terrain_profile = if *rfprop {
        None
    } else {
        Some(
            Profile::builder()
                .start(start.0)
                .start_alt(start.1)
                .max_step(*step_size)
                .earth_curve(*earth_curve)
                .normalize(*normalize)
                .end(dest.0)
                .end_alt(dest.1)
                .build(&tile_src)?,
        )
    };

    match cmd {
        CliCmd::Csv => print_csv(terrain_profile, cli.clone()),
        CliCmd::Plot => plot_ascii(terrain_profile.unwrap()),
        CliCmd::Json => print_json(terrain_profile.unwrap()),
    }
}

/// # Example with gnuplot
///
/// ```sh
/// cargo run -- --srtm-dir=data/nasadem/3arcsecond/ --max-step=90 --earth-curve --normalize --start=0,0,100 --dest=0,1,0 csv | tr ',' ' ' > ~/.tmp/plot && gnuplot -p -e "plot for [col=4:5] '~/.tmp/plot' using 1:col with lines"
/// ```
fn print_csv(profile: Option<Profile<f64>>, cli: Cli) -> Result<(), AnyError> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "Distance,Longitude,Latitude,Los,Elevation")?;
    if let Some(profile) = profile {
        for (((elevation, point), los), distance) in profile
            .terrain_elev_m
            .iter()
            .zip(profile.great_circle.iter())
            .zip(profile.los_elev_m.iter())
            .zip(profile.distances_m.iter())
        {
            let longitude = point.x();
            let latitude = point.y();
            writeln!(
                stdout,
                "{distance},{longitude},{latitude},{los},{elevation}",
            )?;
        }
    } else {
        rfprop::init(Path::new("/Volumes/s3/3-arcsecond/bsdf/"), false)?;
        let profile = rfprop::terrain_profile(
            cli.start.0.y,
            cli.start.0.x,
            cli.start.1,
            cli.dest.0.y,
            cli.dest.0.x,
            cli.dest.1,
            900e6,
            cli.normalize,
        );
        for ((distance, los), elevation) in profile
            .distance
            .iter()
            .zip(profile.los.iter())
            .zip(profile.terrain.iter())
        {
            writeln!(stdout, "{distance},0,0,{los},{elevation}",)?;
        }
    }
    Ok(())
}

fn plot_ascii(profile: Profile<f64>) -> Result<(), AnyError> {
    let plot_data: Vec<(f32, f32)> = profile
        .terrain_elev_m
        .iter()
        .enumerate()
        .map(|(idx, elev)| (f32::from(idx as u16), *elev as f32))
        .collect();
    Chart::new(300, 150, 0.0, plot_data.len() as f32)
        .lineplot(&Shape::Lines(&plot_data))
        .display();
    Ok(())
}

fn print_json(profile: Profile<f64>) -> Result<(), AnyError> {
    #[derive(Serialize)]
    struct JsonEntry {
        location: [f64; 2],
        elevation: f64,
    }

    let reshaped: Vec<JsonEntry> = profile
        .great_circle
        .iter()
        .zip(profile.terrain_elev_m.iter())
        .map(|(point, elev)| JsonEntry {
            location: [point.x(), point.y()],
            elevation: *elev,
        })
        .collect();
    let json = serde_json::to_string(&reshaped)?;
    println!("{}", json);
    Ok(())
}
