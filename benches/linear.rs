use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sais::sais;

pub fn criterion_benchmark(c: &mut Criterion) {
    // Build a pseudo-random byte array so there are interesting patterns, but do it
    // deterministically so different benchmark runs are comparable. The PRNG doesn't need to
    // satisfy statistical tests, so just use a simple one.
    let mut xorshift_state = 0u32;
    let xorshift = std::iter::from_fn(move || {
        xorshift_state ^= xorshift_state << 13;
        xorshift_state ^= xorshift_state >> 17;
        xorshift_state ^= xorshift_state << 5;
        Some(xorshift_state)
    });

    let max_shift = 20;
    let input: Vec<u8> = xorshift
        .flat_map(u32::to_le_bytes)
        .take(1 << max_shift)
        .collect();

    let mut group = c.benchmark_group("sais u8");
    for size in (10..=max_shift).map(|shift| 1 << shift) {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| sais(black_box(&input[input.len() - size..])));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
