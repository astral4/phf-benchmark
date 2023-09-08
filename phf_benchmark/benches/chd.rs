use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use phf_benchmark::*;
use phf_chd::MapGenerator;
use phf_shared::hash::AHasher;
use phf_shared::{PhfMap, FIXED_SEED};
use rand::distributions::Standard;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn eval_chd(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval_chd");

    let mut samples = SmallRng::seed_from_u64(FIXED_SEED).sample_iter(Standard);

    group.bench_function("eval_chd_1k", |b| {
        b.iter_batched(
            || samples.next().unwrap(),
            |x| black_box(MAP_CHD_1000.get_entry(&x)),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("eval_chd_10k", |b| {
        b.iter_batched(
            || samples.next().unwrap(),
            |x| black_box(MAP_CHD_10000.get_entry(&x)),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn build_chd(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_chd");

    for (name, size) in [("1k", 1000), ("10k", 10000)] {
        group.bench_function(format!("build_chd_{name}"), |b| {
            b.iter_batched(
                || {
                    SmallRng::seed_from_u64(FIXED_SEED)
                        .sample_iter(Standard)
                        .take(size)
                },
                |x| black_box(MapGenerator::<u64, u64, AHasher>::from(x)),
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches_chd, eval_chd, build_chd);
criterion_main!(benches_chd);
