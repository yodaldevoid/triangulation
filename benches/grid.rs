use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use triangulation::{Delaunay, Point};

fn bench_grid(count: usize) -> Delaunay {
    let mut points = Vec::with_capacity(count);

    let size = (count as f32).sqrt() as usize;

    for x in 0..size {
        for y in 0..size {
            points.push(Point::new(x as f32 * 10.0, y as f32 * 10.0));
        }
    }

    Delaunay::new(&points).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let bench = |b: &mut Bencher, &&count: &&usize| b.iter(|| bench_grid(count));

    let counts = &[100, 1000, 10_000];
    c.bench_function_over_inputs("grid", bench, counts);

    let counts = &[100_000, 200_000, 500_000, 1_000_000];
    Criterion::default()
        .configure_from_args()
        .sample_size(10)
        .bench_function_over_inputs("grid", bench, counts);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
