use std::str::FromStr;

use criterion::{criterion_group, criterion_main, Criterion};

use srex::srecord::*;

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

fn bench_from_str(c: &mut Criterion) {
    let mut srecord_str = String::new();
    let num_rows = 1000000;
    srecord_str.reserve("S113000000000000000000000000000000000000EC".len() * num_rows);
    for i in 0..num_rows {
        let address = i * 16;
        srecord_str
            .push_str(format!("S113{address:04X}00000000000000000000000000000000EC").as_str());
    }

    let mut group = c.benchmark_group("from_str");
    group.bench_with_input("from_str", srecord_str.as_str(), |b, s| {
        b.iter(|| SRecordFile::from_str(s));
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_calculate_checksum, bench_from_str
}
criterion_main!(benches);
