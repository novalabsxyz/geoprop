mod options;

use anyhow::Error as AnyError;
use clap::Parser;
use options::{Cli, Command as CliCmd};
use serde::Serialize;
use std::{io::Write, path::Path};
use terrain::{
    geo::{point, Point},
    Profile, TileMode, Tiles,
};
use textplots::{Chart, Plot, Shape};

fn main() -> Result<(), AnyError> {
    let cli = Cli::parse();
    let Cli {
        tile_dir,
        rfprop,
        max_step,
        earth_curve,
        normalize,
        start,
        dest,
        cmd,
    } = cli;

    env_logger::init();

    let terrain_profile: CommonProfile = if rfprop {
        rfprop::init(Path::new(&tile_dir), false)?;
        rfprop::terrain_profile(
            cli.start.0.y,
            cli.start.0.x,
            cli.start.1,
            cli.dest.0.y,
            cli.dest.0.x,
            cli.dest.1,
            900e6,
            cli.normalize,
        )
        .into()
    } else {
        let tile_src = Tiles::new(tile_dir, TileMode::MemMap)?;
        Profile::builder()
            .start(start.0)
            .start_alt(start.1)
            .max_step(max_step)
            .earth_curve(earth_curve)
            .normalize(normalize)
            .end(dest.0)
            .end_alt(dest.1)
            .build(&tile_src)?
            .into()
    };

    match cmd {
        CliCmd::Csv => print_csv(terrain_profile),
        CliCmd::Plot => plot_ascii(terrain_profile),
        CliCmd::Json => print_json(terrain_profile),
        CliCmd::Tia => print_tia(terrain_profile),
    }
}

/// # Example with gnuplot
///
/// ```sh
/// cargo run -- --srtm-dir=data/nasadem/3arcsecond/ --max-step=90 --earth-curve --normalize --start=0,0,100 --dest=0,1,0 csv | tr ',' ' ' > ~/.tmp/plot && gnuplot -p -e "plot for [col=4:5] '~/.tmp/plot' using 1:col with lines"
/// ```
fn print_csv(profile: CommonProfile) -> Result<(), AnyError> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "Distance,Longitude,Latitude,Los,Elevation")?;
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
    Ok(())
}

fn plot_ascii(profile: CommonProfile) -> Result<(), AnyError> {
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

fn print_json(profile: CommonProfile) -> Result<(), AnyError> {
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

fn print_tia(profile: CommonProfile) -> Result<(), AnyError> {
    let tia = terrain_intersection_area(
        profile.distances_m.last().unwrap() / profile.distances_m.len() as f64,
        &profile.los_elev_m,
        &profile.terrain_elev_m,
    );
    println!("{tia} m²");
    Ok(())
}

/// Calculate the positive area of intersection, in m², between the
/// profile (terrain) and the line of sight.
fn terrain_intersection_area(step_size_m: f64, los_vec: &[f64], profile: &[f64]) -> f64 {
    los_vec
        .iter()
        .zip(profile.iter())
        .map(|(los, prof)| (prof - los).max(0.0) * step_size_m)
        .sum::<f64>()
}

/// A common represention of both native and rfprop profiles.
struct CommonProfile {
    great_circle: Vec<Point<f64>>,
    distances_m: Vec<f64>,
    los_elev_m: Vec<f64>,
    terrain_elev_m: Vec<f64>,
}

impl From<Profile<f64>> for CommonProfile {
    fn from(
        Profile {
            distances_m,
            great_circle,
            los_elev_m,
            terrain_elev_m,
            ..
        }: Profile<f64>,
    ) -> Self {
        Self {
            distances_m,
            great_circle,
            los_elev_m,
            terrain_elev_m,
        }
    }
}

impl From<rfprop::TerrainProfile> for CommonProfile {
    fn from(
        rfprop::TerrainProfile {
            mut distance,
            los,
            terrain,
            ..
        }: rfprop::TerrainProfile,
    ) -> Self {
        distance.iter_mut().for_each(|val| *val *= 1000.0);
        let great_circle = std::iter::repeat(point!(x: 0.0, y:0.0))
            .take(terrain.len())
            .collect();
        Self {
            distances_m: distance,
            great_circle,
            los_elev_m: los,
            terrain_elev_m: terrain,
        }
    }
}
