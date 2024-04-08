use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use srex::srecord::calculate_checksum;

fn bench_calculate_checksum(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculate_checksum");

    let byte_count: u8 = 11;
    let address: u32 = 0;
    let data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let input = (byte_count, address, &data);
    group.bench_with_input("calculate_checksum", &input, |b, i| {
        b.iter(|| calculate_checksum(i.0, i.1, i.2))
    });
}

criterion_group!(calculate_checksum_slkjkldsf, bench_calculate_checksum,);
criterion_main!(calculate_checksum_slkjkldsf);
