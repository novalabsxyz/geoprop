#![allow(clippy::excessive_precision)]

use approx::assert_relative_eq;
use criterion::{criterion_group, criterion_main, Criterion};
use geo::{
    algorithm::VincentyDistance,
    coord,
    geometry::{Coord, Point},
};
use std::{env, path::PathBuf};
use terrain::{Profile, TileMode, Tiles};

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn three_arcsecond_dir() -> std::path::PathBuf {
    let three_arcsecond_dir_str = env::var("SRTM_3ARCSEC_DIR").expect(
        "'export SRTM_3ARCSEC_DIR=/path/to/3arcsecond/hgt/files' before running benchmarks",
    );
    PathBuf::from(three_arcsecond_dir_str)
}

fn distance(start: Coord<f32>, end: Coord<f32>) -> f32 {
    Point(start).vincenty_distance(&end.into()).unwrap()
}

fn memmap_terrain_profile(c: &mut Criterion) {
    let tile_source = Tiles::new(three_arcsecond_dir(), TileMode::MemMap).unwrap();

    let mut group = c.benchmark_group("Terrain Profile - MemMap");

    let _2_4km @ (start, end) = (
        coord!(x:-117.3519964316712f32, y:52.30693919915002f32),
        coord!(x:-117.3165476765753f32, y:52.30866462880422f32),
    );
    assert_relative_eq!(2425.2756, distance(start, end), epsilon = 0.1);

    let _67km @ (start, end) = (
        coord!(x:22.02060050752248f32, y:17.32531643138395f32),
        coord!(x:22.6498898241764f32, y:17.31391991428638f32),
    );
    assert_relative_eq!(66907.47, distance(start, end), epsilon = 0.1);

    let _103km @ (start, end) = (
        coord!(x:95.15915866746103f32, y:38.89938117857166f32),
        coord!(x:94.615374082193f32, y:39.72746075951511f32),
    );
    assert_relative_eq!(103205.28, distance(start, end), epsilon = 0.1);

    // Distance between each elevation sample
    let _90m = 90.0;

    group.bench_with_input(
        "2.4 km",
        &(&tile_source, _90m, _2_4km),
        |b, (t, d, (s, e))| b.iter(|| Profile::new(*s, *d, *e, t).unwrap()),
    );

    group.bench_with_input(
        "67 km",
        &(&tile_source, _90m, _67km),
        |b, (t, d, (s, e))| b.iter(|| Profile::new(*s, *d, *e, t).unwrap()),
    );

    group.bench_with_input(
        "103 km",
        &(&tile_source, _90m, _103km),
        |b, (t, d, (s, e))| b.iter(|| Profile::new(*s, *d, *e, t).unwrap()),
    );
}

criterion_group!(benches, memmap_terrain_profile);
criterion_main!(benches);
