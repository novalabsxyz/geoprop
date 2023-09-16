use criterion::{criterion_group, criterion_main, Criterion};
use geo::geometry::Coord;
use std::env;
use terrain::{Profile, TileMode, TileSource};

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn three_arcsecond_dir() -> std::path::PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "data",
        "nasadem",
        "3arcsecond",
    ]
    .iter()
    .collect()
}

fn terrain_profile(c: &mut Criterion) {
    let mut group = c.benchmark_group("Terrain Profile");

    let start = Coord {
        x: -71.30830716441369,
        y: 44.28309806603165,
    };

    let end = Coord {
        x: -71.2972073283768,
        y: 44.25628098424278,
    };

    let tile_source = TileSource::new(three_arcsecond_dir(), TileMode::MemMap).unwrap();
    let _90m = 90.0;

    group.bench_with_input(
        "short",
        &(tile_source, _90m, start, end),
        |b, (t, d, s, e)| b.iter(|| Profile::new(*s, *d, *e, t).unwrap()),
    );
}

criterion_group!(benches, terrain_profile);
criterion_main!(benches);
