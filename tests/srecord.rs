use std::{fs, str::FromStr};

use srex::srecord::*;

#[test]
fn test_srecord_file_new() {
    let srecord_file = SRecordFile::new();
    assert_eq!(srecord_file.header_data, None);
    assert_eq!(srecord_file.data_chunks, []);
    assert_eq!(srecord_file.start_address, None);
}

#[test]
fn test_srecord_file_default() {
    let srecord_file = Option::<SRecordFile>::None.unwrap_or_default();
    assert_eq!(srecord_file.header_data, None);
    assert_eq!(srecord_file.data_chunks, []);
    assert_eq!(srecord_file.start_address, None);
}

#[test]
fn test_srecord_file_from_str() {
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    assert_eq!(
        srecord_file.header_data.unwrap(),
        Vec::<u8>::from([0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00])
    );
    assert_eq!(
        srecord_file.data_chunks,
        Vec::<DataChunk>::from([DataChunk {
            address: 0x0000,
            data: Vec::<u8>::from([
                0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF, 0xF0, 0x7C, 0x6C,
                0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63, 0x00, 0x00,
                0x4B, 0xFF, 0xFF, 0xE5, 0x39, 0x80, 0x00, 0x00, 0x7D, 0x83, 0x63, 0x78, 0x80, 0x01,
                0x00, 0x14, 0x38, 0x21, 0x00, 0x10, 0x7C, 0x08, 0x03, 0xA6, 0x4E, 0x80, 0x00, 0x20,
                0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x2E, 0x0A, 0x00,
            ])
        }]),
    );
    assert_eq!(srecord_file.start_address, Some(0));
}

#[test]
fn test_parse_srecord_unsorted_data() {
    let srecord_str = fs::read_to_string("tests/srec_files/unsorted.s28").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    assert_eq!(srecord_file.header_data, Some(Vec::<u8>::new()));
    assert_eq!(
        srecord_file.data_chunks,
        [
            DataChunk {
                address: 0x01,
                data: Vec::<u8>::from([0x01, 0x02, 0x03])
            },
            DataChunk {
                address: 0x05,
                data: Vec::<u8>::from([0x05])
            },
        ]
    );
    assert_eq!(srecord_file.start_address, Some(0x00));
}

#[test]
fn test_parse_srecord_multiple_data_chunks() {
    let srecord_str = fs::read_to_string("tests/srec_files/data_chunks.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();
    assert_eq!(srecord_file.data_chunks.len(), 16);
    for (i, data_chunk) in srecord_file.data_chunks.iter().enumerate() {
        let expected_start_address = 0x1000 * i as u64;
        let expected_end_address = expected_start_address + 16;
        assert_eq!(data_chunk.start_address(), expected_start_address);
        assert_eq!(data_chunk.end_address(), expected_end_address);
    }
}
#[test]
fn test_parse_srecord_error() {
    assert!(SRecordFile::from_str("S").is_err());
}

#[test]
fn test_parse_srecord_multiple_start_addresses() {
    let srecord_str = fs::read_to_string("tests/srec_files/multiple_start_addresses.s19").unwrap();
    assert!(SRecordFile::from_str(&srecord_str).is_err());
}

#[test]
fn test_parse_srecord_invalid_record_count() {
    let srecord_str = fs::read_to_string("tests/srec_files/invalid_record_count.s19").unwrap();
    assert!(SRecordFile::from_str(&srecord_str).is_err());
}

#[test]
fn test_srecord_file_index() {
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    assert_eq!(srecord_file[0x00], 0x7C);
    assert_eq!(srecord_file[0x01], 0x08);
    assert_eq!(srecord_file[0x3F], 0x6F);

    assert_eq!(srecord_file[0x00..0x01], [0x7C]);
    assert_eq!(srecord_file[0x00..0x02], [0x7C, 0x08]);
    assert_eq!(srecord_file[0x43..0x46], [0x2E, 0x0A, 0x00]);
}

#[test]
#[should_panic]
fn test_srecord_file_index_error_1() {
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    let x = srecord_file[0xFF];
    println!("This should not be printed: {x:#02X}");
}

#[test]
#[should_panic]
fn test_srecord_file_index_error_2() {
    let srecord_str = fs::read_to_string("tests/srec_files/offset_data.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    let x = &srecord_file[0x00..0x2F];
    let x0 = x[0];
    println!("This should not be printed: {x0:#02X}");
}

#[test]
#[should_panic]
fn test_srecord_file_index_error_3() {
    let srecord_str = fs::read_to_string("tests/srec_files/offset_data.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    let x = &srecord_file[0x20..0x5F];
    let x0 = x[0];
    println!("This should not be printed: {x0:#02X}");
}

#[test]
fn test_srecord_file_index_mut() {
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s19").unwrap();
    let mut srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    assert_eq!(srecord_file[0x00], 0x7C);
    srecord_file[0x00] = 0xFF;
    assert_eq!(srecord_file[0x00], 0xFF);
}

#[test]
#[should_panic]
fn test_srecord_file_index_mut_error() {
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s19").unwrap();
    let mut srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    srecord_file[0xFF] = 0x00;
    let x = srecord_file[0xFF];
    println!("This should not be printed: {x:#02X}");
}

#[test]
fn test_serialize_from_str() {
    // Test that serializing an SRecordFile parsed by from_str results in the same string.
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s37").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();
    let mut serialized_str = String::new();
    for record in srecord_file.iter_records(0x1C) {
        serialized_str.push_str(record.serialize().as_str());
        serialized_str.push('\n');
    }
    assert_eq!(serialized_str, srecord_str);
}
