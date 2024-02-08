mod options;

use anyhow::Error as AnyErr;
use clap::Parser;
use itm::{Climate, ModeVariability, Polarization};
use options::{Cli, LatLonAlt};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use terrain::{Profile, TileMode, Tiles};

// Example for 900 MHz across the Grand Canyon.
// ```
// cargo run --release --example path -- --tile-dir=/Volumes/s3/nasadem/3-arcsecond/srtm/ --start=36.00413897612008,-112.2797569088778,3 --frequency=900e6 --end=36.20334730019485,-112.1230717397408,3
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
    let epsilon = 15.;
    let f_hz = frequency;
    let location = 50.0;
    let mdvar = ModeVariability::Accidental;
    let n0 = 301.;
    let pol = Polarization::Vertical;
    let sigma = 0.005;
    let situation = 50.0;
    let step_size_m = profile.distances_m[1];
    let terrain = profile.terrain_elev_m;
    let time = 50.0;

    let t0 = std::time::Instant::now();

    let loss_path_db = (1..terrain.len())
        .into_par_iter()
        .map(|end_idx| {
            let terrain = &terrain[..=end_idx];
            itm::p2p(
                start_alt.into(),
                end_alt.into(),
                step_size_m.into(),
                terrain,
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
            )
        })
        .collect::<Result<Vec<f64>, _>>()?;

    let itm_p2p_runtime = t0.elapsed();

    let total_distance_m = profile.distances_m.last().unwrap();
    let fspl = fspl(*total_distance_m, frequency);

    eprintln!("profile runtime:  {profile_runtime:?}");
    eprintln!("itm runtime:      {itm_p2p_runtime:?}");
    eprintln!("distance:         {total_distance_m} m");
    eprintln!("fspl:             {fspl} dB");
    println!("distance_m,rx_elev_m,atten_db");
    for i in 0..loss_path_db.len() {
        let distance_m = profile.distances_m[i + 1];
        let rx_elev_m = terrain[i + 1] + end_alt;
        let atten_db = -loss_path_db[i];
        println!("{distance_m},{rx_elev_m},{atten_db}");
    }

    Ok(())
}

fn fspl(meters: f32, freq: f32) -> f32 {
    20.0 * meters.log10() + 20.0 * freq.log10() - 147.55
}
