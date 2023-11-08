mod options;

use anyhow::Error as AnyErr;
use clap::Parser;
use itm::{Climate, ModeVariability, Polarization};
use options::{Cli, LatLonAlt};
use terrain::{Profile, TileMode, Tiles};

// Example for 900 MHz across black rock desert.
// ```
// cargo run --example p2p -- --tile-dir=/path/to/nasadem/3-arcsecond/hgt/tiles/ --start=40.885629,-119.065844,10 --frequency=900e6 --end=40.904691,-119.043429,10
// ```
fn main() -> Result<(), AnyErr> {
    let Cli {
        tile_dir,
        max_step,
        start: LatLonAlt(start_coord, start_alt),
        end: LatLonAlt(end_coord, end_alt),
        frequency,
    } = Cli::parse();

    let tiles = Tiles::new(tile_dir, TileMode::MemMap)?;
    let t0 = std::time::Instant::now();
    let profile = Profile::<f32>::builder()
        .start(start_coord)
        .start_alt(start_alt)
        .max_step(max_step)
        .end(end_coord)
        .end_alt(end_alt)
        .build(&tiles)?;
    let profile_runtime = t0.elapsed();

    let climate = Climate::Desert;
    let n0 = 301.;
    let f_hz = frequency;
    let pol = Polarization::Vertical;
    let epsilon = 15.;
    let sigma = 0.005;
    let mdvar = ModeVariability::Accidental;
    let time = 50.0;
    let location = 50.0;
    let situation = 50.0;
    let step_size_m = profile.distances_m[1];
    let terrain = profile.terrain_elev_m;
    let t0 = std::time::Instant::now();
    let attenuation_db = itm::p2p(
        start_alt.into(),
        end_alt.into(),
        step_size_m.into(),
        &terrain,
        climate,
        n0,
        f_hz.into(),
        pol,
        epsilon,
        sigma,
        mdvar,
        time,
        location,
        situation,
    )?;
    let itm_p2p_runtime = t0.elapsed();

    let total_distance_m = profile.distances_m.last().unwrap();
    let fspl = fspl(*total_distance_m, frequency);

    println!("profile runtime: {profile_runtime:?}");
    println!("itm runtime:     {itm_p2p_runtime:?}");
    println!("distance:        {total_distance_m} m");
    println!("fspl:            {fspl} dB");
    println!("attenuation:     {attenuation_db} dB");

    Ok(())
}

fn fspl(meters: f32, freq: f32) -> f32 {
    20.0 * meters.log10() + 20.0 * freq.log10() - 147.55
}
