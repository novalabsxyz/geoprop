use assert_approx_eq::assert_approx_eq;
use criterion::{criterion_group, criterion_main, Criterion};
use geo::{
    algorithm::vincenty_distance::VincentyDistance,
    coord,
    geometry::{Coord, Point},
};
use std::{env, path::PathBuf};
use terrain::{Profile, TileMode, TileSource};

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

fn distance(start: Coord<f64>, end: Coord<f64>) -> f64 {
    Point(start).vincenty_distance(&end.into()).unwrap()
}

fn memmap_terrain_profile(c: &mut Criterion) {
    let tile_source = TileSource::new(three_arcsecond_dir(), TileMode::MemMap).unwrap();

    let mut group = c.benchmark_group("Terrain Profile - MemMap");

    let _2_4km @ (start, end) = (
        coord!(x:-117.3519964316712, y:52.30693919915002),
        coord!(x:-117.3165476765753, y:52.30866462880422),
    );
    assert_approx_eq!(2.4254282742170944e3, distance(start, end));

    let _67km @ (start, end) = (
        coord!(x:22.02060050752248, y:17.32531643138395),
        coord!(x:22.6498898241764, y:17.31391991428638),
    );
    assert_approx_eq!(66.90763248696443e3, distance(start, end));

    let _103km @ (start, end) = (
        coord!(x:95.15915866746103, y:38.89938117857166),
        coord!(x:94.615374082193, y:39.72746075951511),
    );
    assert_approx_eq!(103204.81565942978, distance(start, end));

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
