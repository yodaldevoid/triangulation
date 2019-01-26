use criterion::{criterion_group, criterion_main, Bencher, Criterion};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use triangulation::{Delaunay, Point};

fn bench_uniform(count: usize) -> Delaunay {
    let mut rng = StdRng::seed_from_u64(1337);
    let mut points = Vec::with_capacity(count);

    for _ in 0..count {
        let x = rng.gen_range(0.0, 10000.0);
        let y = rng.gen_range(0.0, 10000.0);
        points.push(Point::new(x, y));
    }

    Delaunay::new(&points).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let bench = |b: &mut Bencher, &&count: &&usize| b.iter(|| bench_uniform(count));

    let counts = &[100, 1000, 10_000];
    c.bench_function_over_inputs("uniform", bench, counts);

    let counts = &[100_000, 200_000, 500_000, 1_000_000];
    Criterion::default()
        .configure_from_args()
        .sample_size(10)
        .bench_function_over_inputs("uniform", bench, counts);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
