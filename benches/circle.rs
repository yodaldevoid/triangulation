use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use triangulation::{Delaunay, Point};

fn bench_circle(count: usize) -> Delaunay {
    let mut points = Vec::with_capacity(count);

    for i in 0..count {
        let angle = i as f32 / count as f32 * 2.0 * std::f32::consts::PI;
        let (sin, cos) = angle.sin_cos();
        points.push(Point::new(cos * 1000.0, sin * 1000.0));
    }

    Delaunay::new(&points).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let bench = |b: &mut Bencher, &&count: &&usize| b.iter(|| bench_circle(count));

    let counts = &[100, 1000, 10_000];
    c.bench_function_over_inputs("circle", bench, counts);

    let counts = &[100_000, 200_000, 500_000, 1_000_000];
    Criterion::default()
        .configure_from_args()
        .sample_size(10)
        .bench_function_over_inputs("circle", bench, counts);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
