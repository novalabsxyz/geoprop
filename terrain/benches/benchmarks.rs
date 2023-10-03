#![allow(clippy::excessive_precision)]

use criterion::{criterion_group, criterion_main, Criterion};
use geo::coord;
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

fn memmap_terrain_profile_f32(c: &mut Criterion) {
    let tile_source = Tiles::new(three_arcsecond_dir(), TileMode::MemMap).unwrap();

    let mut group = c.benchmark_group("Profile<f32>");

    let _2_4km = (
        coord!(x:-117.3519964316712f32, y:52.30693919915002f32),
        coord!(x:-117.3165476765753f32, y:52.30866462880422f32),
    );

    let _67km = (
        coord!(x:22.02060050752248f32, y:17.32531643138395f32),
        coord!(x:22.6498898241764f32, y:17.31391991428638f32),
    );

    let _103km = (
        coord!(x:95.15915866746103f32, y:38.89938117857166f32),
        coord!(x:94.615374082193f32, y:39.72746075951511f32),
    );

    // Distance between each elevation sample
    let _90m = 90.0;

    group.bench_with_input(
        "2.4_km",
        &(&tile_source, _90m, _2_4km),
        |b, (t, d, (s, e))| {
            b.iter(|| {
                Profile::builder()
                    .start(*s)
                    .max_step(*d)
                    .end(*e)
                    .earth_curve(true)
                    .normalize(true)
                    .build(t)
                    .unwrap()
            })
        },
    );

    group.bench_with_input(
        "67_km",
        &(&tile_source, _90m, _67km),
        |b, (t, d, (s, e))| {
            b.iter(|| {
                Profile::builder()
                    .start(*s)
                    .max_step(*d)
                    .end(*e)
                    .build(t)
                    .unwrap()
            })
        },
    );

    group.bench_with_input(
        "103_km",
        &(&tile_source, _90m, _103km),
        |b, (t, d, (s, e))| {
            b.iter(|| {
                Profile::builder()
                    .start(*s)
                    .max_step(*d)
                    .end(*e)
                    .build(t)
                    .unwrap()
            })
        },
    );
}

fn memmap_terrain_profile_f64(c: &mut Criterion) {
    let tile_source = Tiles::new(three_arcsecond_dir(), TileMode::MemMap).unwrap();

    let mut group = c.benchmark_group("Profile<f64>");

    let _2_4km = (
        coord!(x:-117.3519964316712f64, y:52.30693919915002f64),
        coord!(x:-117.3165476765753f64, y:52.30866462880422f64),
    );

    let _67km = (
        coord!(x:22.02060050752248f64, y:17.32531643138395f64),
        coord!(x:22.6498898241764f64, y:17.31391991428638f64),
    );

    let _103km = (
        coord!(x:95.15915866746103f64, y:38.89938117857166f64),
        coord!(x:94.615374082193f64, y:39.72746075951511f64),
    );

    // Distance between each elevation sample
    let _90m = 90.0;

    group.bench_with_input(
        "2.4_km",
        &(&tile_source, _90m, _2_4km),
        |b, (t, d, (s, e))| {
            b.iter(|| {
                Profile::builder()
                    .start(*s)
                    .max_step(*d)
                    .end(*e)
                    .earth_curve(true)
                    .normalize(true)
                    .build(t)
                    .unwrap()
            })
        },
    );

    group.bench_with_input(
        "67_km",
        &(&tile_source, _90m, _67km),
        |b, (t, d, (s, e))| {
            b.iter(|| {
                Profile::builder()
                    .start(*s)
                    .max_step(*d)
                    .end(*e)
                    .build(t)
                    .unwrap()
            })
        },
    );

    group.bench_with_input(
        "103_km",
        &(&tile_source, _90m, _103km),
        |b, (t, d, (s, e))| {
            b.iter(|| {
                Profile::builder()
                    .start(*s)
                    .max_step(*d)
                    .end(*e)
                    .build(t)
                    .unwrap()
            })
        },
    );
}

criterion_group!(
    benches,
    memmap_terrain_profile_f32,
    memmap_terrain_profile_f64
);
criterion_main!(benches);
