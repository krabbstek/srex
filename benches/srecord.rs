use std::str::FromStr;

use criterion::{criterion_group, criterion_main, Criterion};

use srex::srecord::utils::calculate_checksum;
use srex::srecord::SRecordFile;

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
    let num_rows: u32 = 1000000;
    srecord_str.reserve("S315000000000000000000000000000000000000EC\n".len() * num_rows as usize);
    for i in 0..num_rows {
        let address = i * 16;
        let checksum = calculate_checksum(0x15, address, &[]);
        srecord_str.push_str(
            format!("S315{address:08X}00000000000000000000000000000000{checksum:02X}\n").as_str(),
        );
    }

    let mut group = c.benchmark_group("Sequential data");
    group.bench_with_input("1M 16 byte", srecord_str.as_str(), |b, s| {
        b.iter(|| SRecordFile::from_str(s).unwrap());
    });

    let mut srecord_str = String::new();
    let num_rows: u32 = 500000;
    srecord_str.reserve(
        "S32500000000000000000000000000000000000000000000000000000000000000000000XX\n".len()
            * num_rows as usize,
    );
    for i in 0..num_rows {
        let address = i * 32;
        let checksum = calculate_checksum(0x25, address, &[]);
        srecord_str
            .push_str(format!("S325{address:08X}0000000000000000000000000000000000000000000000000000000000000000{checksum:02X}\n").as_str());
    }

    group.bench_with_input("500k 32 byte", srecord_str.as_str(), |b, s| {
        b.iter(|| SRecordFile::from_str(s).unwrap());
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_calculate_checksum, bench_from_str,
}
criterion_main!(benches);
