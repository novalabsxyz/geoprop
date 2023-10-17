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

    let p_2_4km = (
        coord!(x:-117.351_996_431_671_2_f32, y:52.306_939_199_150_02_f32),
        coord!(x:-117.316_547_676_575_3_f32, y:52.308_664_628_804_22_f32),
    );

    let p_67km = (
        coord!(x:22.020_600_507_522_48_f32, y:17.325_316_431_383_95_f32),
        coord!(x:22.649_889_824_176_4_f32, y:17.313_919_914_286_38_f32),
    );

    let p_103km = (
        coord!(x:95.159_158_667_461_03_f32, y:38.899_381_178_571_66_f32),
        coord!(x:94.615_374_082_193_f32, y:39.727_460_759_515_11_f32),
    );

    // Distance between each elevation sample
    let d_90m = 90.0;

    group.bench_with_input(
        "2.4_km",
        &(&tile_source, d_90m, p_2_4km),
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
            });
        },
    );

    group.bench_with_input(
        "67_km",
        &(&tile_source, d_90m, p_67km),
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
            });
        },
    );

    group.bench_with_input(
        "103_km",
        &(&tile_source, d_90m, p_103km),
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
            });
        },
    );
}

fn memmap_terrain_profile_f64(c: &mut Criterion) {
    let tile_source = Tiles::new(three_arcsecond_dir(), TileMode::MemMap).unwrap();

    let mut group = c.benchmark_group("Profile<f64>");

    let p_2_4km = (
        coord!(x:-117.351_996_431_671_2_f64, y:52.306_939_199_150_02_f64),
        coord!(x:-117.316_547_676_575_3_f64, y:52.308_664_628_804_22_f64),
    );

    let p_67km = (
        coord!(x:22.020_600_507_522_48_f64, y:17.325_316_431_383_95_f64),
        coord!(x:22.649_889_824_176_4_f64, y:17.313_919_914_286_38_f64),
    );

    let p_103km = (
        coord!(x:95.159_158_667_461_03_f64, y:38.899_381_178_571_66_f64),
        coord!(x:94.615_374_082_193_f64, y:39.727_460_759_515_11_f64),
    );

    // Distance between each elevation sample
    let d_90m = 90.0;

    group.bench_with_input(
        "2.4_km",
        &(&tile_source, d_90m, p_2_4km),
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
            });
        },
    );

    group.bench_with_input(
        "67_km",
        &(&tile_source, d_90m, p_67km),
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
            });
        },
    );

    group.bench_with_input(
        "103_km",
        &(&tile_source, d_90m, p_103km),
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
            });
        },
    );
}

criterion_group!(
    benches,
    memmap_terrain_profile_f32,
    memmap_terrain_profile_f64
);
criterion_main!(benches);
