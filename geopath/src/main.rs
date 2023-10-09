mod options;

use anyhow::Error as AnyError;
use clap::Parser;
use itertools::Itertools;
use num_traits::{AsPrimitive, Float, FromPrimitive};
use options::{Cli, Command as CliCmd};
use propah::fresnel::FresnelZone;
use rfprop::TerrainProfile;
use serde::Serialize;
use std::{io::Write, path::Path};
use terrain::{
    geo::{coord, point, CoordFloat, Point},
    Profile, TileMode, Tiles,
};
use textplots::{Chart, Plot, Shape};

fn main() -> Result<(), AnyError> {
    let cli = Cli::parse();
    let Cli {
        tile_dir,
        rfprop,
        use_f32,
        max_step,
        earth_curve,
        normalize,
        start,
        dest,
        fresnel_zone,
        frequency,
        cmd,
    } = cli;

    env_logger::init();

    if use_f32 {
        type C = f32;

        let terrain_profile: CommonProfile<C> = if rfprop {
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
            let start_point = coord!(x: start.0.x as C, y: start.0.y as C);
            let start_alt = start.1 as C;
            let max_step = max_step as C;
            let dest_point = coord!(x: dest.0.x as C, y: dest.0.y as C);
            let dest_alt = dest.1 as C;

            eprintln!(
                "start_point: {start_point:?}, start_alt: {start_alt}, max_step: {max_step}, dest_point: {dest_point:?}, dest_alt: {dest_alt}"
            );

            let tile_src = Tiles::new(tile_dir, TileMode::MemMap)?;
            let profile = Profile::<C>::builder()
                .start(start_point)
                .start_alt(start_alt)
                .max_step(max_step)
                .earth_curve(earth_curve)
                .normalize(normalize)
                .end(dest_point)
                .end_alt(dest_alt)
                .build(&tile_src)?;

            let fresnel_zone: Vec<C> =
                if let (Some(zone), Some(frequency)) = (fresnel_zone, frequency) {
                    FresnelZone::new(zone, frequency as C, *profile.distances_m.last().unwrap())
                        .iter(profile.distances_m.len())
                        .collect()
                } else {
                    std::iter::repeat(0 as C)
                        .take(profile.distances_m.len())
                        .collect()
                };
            (profile, fresnel_zone).into()
        };

        match cmd {
            CliCmd::Csv => print_csv(terrain_profile),
            CliCmd::Plot => plot_ascii(terrain_profile),
            CliCmd::Json => print_json(terrain_profile),
            CliCmd::Tia => print_tia(terrain_profile),
        }
    } else {
        type C = f64;

        let terrain_profile: CommonProfile<C> = if rfprop {
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
            let profile = Profile::<C>::builder()
                .start(coord!(x: start.0.x, y: start.0.y))
                .start_alt(start.1)
                .max_step(max_step)
                .earth_curve(earth_curve)
                .normalize(normalize)
                .end(coord!(x: dest.0.x, y: dest.0.y))
                .end_alt(dest.1)
                .build(&tile_src)?;
            let fresnel_zone: Vec<C> =
                if let (Some(zone), Some(frequency)) = (fresnel_zone, frequency) {
                    FresnelZone::new(zone, frequency as C, *profile.distances_m.last().unwrap())
                        .iter(profile.distances_m.len())
                        .collect()
                } else {
                    std::iter::repeat(f64::from(0))
                        .take(profile.distances_m.len())
                        .collect()
                };
            (profile, fresnel_zone).into()
        };

        match cmd {
            CliCmd::Csv => print_csv(terrain_profile),
            CliCmd::Plot => plot_ascii(terrain_profile),
            CliCmd::Json => print_json(terrain_profile),
            CliCmd::Tia => print_tia(terrain_profile),
        }
    }
}

/// # Example with gnuplot
///
/// ```sh
/// cargo run -- --srtm-dir=data/nasadem/3arcsecond/ --max-step=90 --earth-curve --normalize --start=0,0,100 --dest=0,1,0 csv | tr ',' ' ' > ~/.tmp/plot && gnuplot -p -e "plot for [col=4:5] '~/.tmp/plot' using 1:col with lines"
/// ```
fn print_csv<T: CoordFloat + std::fmt::Display>(profile: CommonProfile<T>) -> Result<(), AnyError> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "Distance,Longitude,Latitude,LOS,Elevation,Fresnel")?;
    for ((((elevation, point), los), distance), fresnel) in profile
        .terrain_elev_m
        .iter()
        .zip(profile.great_circle.iter())
        .zip(profile.los_elev_m.iter())
        .zip(profile.distances_m.iter())
        .zip(profile.fresnel_zone.iter())
    {
        let longitude = point.x();
        let latitude = point.y();
        let fresnel = *los - *fresnel;
        writeln!(
            stdout,
            "{distance},{longitude},{latitude},{los},{elevation},{fresnel}",
        )?;
    }
    Ok(())
}

fn plot_ascii<T>(profile: CommonProfile<T>) -> Result<(), AnyError>
where
    T: CoordFloat + AsPrimitive<f32>,
{
    let plot_data: Vec<(f32, f32)> = profile
        .terrain_elev_m
        .iter()
        .enumerate()
        .map(|(idx, elev)| (f32::from(idx as u16), elev.as_()))
        .collect();
    Chart::new(300, 150, 0.0, plot_data.len() as f32)
        .lineplot(&Shape::Lines(&plot_data))
        .display();
    Ok(())
}

fn print_json<T>(profile: CommonProfile<T>) -> Result<(), AnyError>
where
    T: CoordFloat + Serialize,
{
    #[derive(Serialize)]
    struct JsonEntry<T> {
        location: [T; 2],
        elevation: T,
    }

    let reshaped: Vec<JsonEntry<T>> = profile
        .great_circle
        .iter()
        .zip(profile.terrain_elev_m.iter())
        .map(|(point, elev)| JsonEntry {
            location: [point.x(), point.y()],
            elevation: *elev,
        })
        .collect();
    let json = serde_json::to_string(&reshaped)?;
    println!("{json}");
    Ok(())
}

fn print_tia<T>(profile: CommonProfile<T>) -> Result<(), AnyError>
where
    T: CoordFloat + FromPrimitive + std::fmt::Display + std::iter::Sum,
{
    let tia = terrain_intersection_area(
        &profile.distances_m,
        &profile.los_elev_m,
        &profile.terrain_elev_m,
    );
    println!("{tia} m²");
    Ok(())
}

/// Calculate the positive area of intersection, in m², between the
/// profile (terrain) and the line of sight.
fn terrain_intersection_area<T>(distances_m: &[T], los_elev_m: &[T], terrain_elev_m: &[T]) -> T
where
    T: Float + FromPrimitive + std::iter::Sum,
{
    let tia_m2 = los_elev_m
        .iter()
        .zip(terrain_elev_m.iter())
        .map(|(los, prof)| (*prof - *los).max(T::zero()))
        .tuple_windows::<(T, T)>()
        .zip(distances_m.iter().tuple_windows::<(&T, &T)>())
        .map(|((h_n1, h_n), (d_n1, d_n))| {
            let dx = (*d_n - *d_n1).abs();
            dx * (h_n + h_n1) / T::from(2).unwrap()
        })
        .sum::<T>();

    // Convert distance from `m^2` to `m*km` to stay compatible with
    // DB assumptions.
    tia_m2
}

/// A common represention of both native and rfprop profiles.
struct CommonProfile<T: CoordFloat> {
    great_circle: Vec<Point<T>>,
    distances_m: Vec<T>,
    los_elev_m: Vec<T>,
    terrain_elev_m: Vec<T>,
    fresnel_zone: Vec<T>,
}

impl<T: CoordFloat> From<(Profile<T>, Vec<T>)> for CommonProfile<T> {
    fn from(
        (
            Profile {
                distances_m,
                great_circle,
                los_elev_m,
                terrain_elev_m,
                ..
            },
            fresnel_zone,
        ): (Profile<T>, Vec<T>),
    ) -> Self {
        Self {
            great_circle,
            distances_m,
            los_elev_m,
            terrain_elev_m,
            fresnel_zone,
        }
    }
}

impl From<TerrainProfile> for CommonProfile<f32> {
    fn from(
        TerrainProfile {
            distance,
            los,
            terrain,
            fresnel,
            ..
        }: TerrainProfile,
    ) -> Self {
        let distances_m = distance.iter().map(|val| *val as f32 * 1000.0).collect();
        let los_elev_m = los.iter().map(|val| *val as f32).collect();
        let terrain_elev_m = los.iter().map(|val| *val as f32).collect();
        let fresnel_zone = fresnel.iter().map(|val| *val as f32).collect();
        let great_circle = std::iter::repeat(point!(x: 0.0, y:0.0))
            .take(terrain.len())
            .collect();
        Self {
            great_circle,
            distances_m,
            los_elev_m,
            terrain_elev_m,
            fresnel_zone,
        }
    }
}

impl From<TerrainProfile> for CommonProfile<f64> {
    fn from(
        TerrainProfile {
            mut distance,
            los,
            terrain,
            fresnel,
            ..
        }: TerrainProfile,
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
            fresnel_zone: fresnel,
        }
    }
}
