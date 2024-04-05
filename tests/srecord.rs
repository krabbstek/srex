use std::{fs, str::FromStr};

use srex::srecord::*;

#[test]
fn test_calculate_checksum() {
    assert_eq!(
        calculate_checksum(
            0x13,
            0x7AF0,
            &[0x0A, 0x0A, 0x0D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        ),
        0x61
    );
    assert_eq!(
        calculate_checksum(
            0x0F,
            0x0000,
            &[0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x20, 0x20, 0x20, 0x20, 0, 0]
        ),
        0x3C
    );
    assert_eq!(
        calculate_checksum(
            0x1F,
            0x0000,
            &[
                0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF, 0xF0, 0x7C, 0x6C,
                0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63, 0x00, 0x00
            ]
        ),
        0x26
    );
    assert_eq!(
        calculate_checksum(
            0x1F,
            0x001C,
            &[
                0x4B, 0xFF, 0xFF, 0xE5, 0x39, 0x80, 0x00, 0x00, 0x7D, 0x83, 0x63, 0x78, 0x80, 0x01,
                0x00, 0x14, 0x38, 0x21, 0x00, 0x10, 0x7C, 0x08, 0x03, 0xA6, 0x4E, 0x80, 0x00, 0x20
            ]
        ),
        0xE9
    );
    assert_eq!(
        calculate_checksum(
            0x11,
            0x0038,
            &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x2E, 0x0A, 0x00]
        ),
        0x42
    );
    assert_eq!(calculate_checksum(0x03, 0x0003, &[]), 0xF9);
    assert_eq!(calculate_checksum(0x03, 0x0000, &[]), 0xFC);
}

#[test]
fn test_parse_record() {
    // Test some simple strings

    let record = parse_record("S007000065666700C6").unwrap();
    assert_eq!(record.record_type, RecordType::S0);
    assert_eq!(record.byte_count, 0x07);
    assert_eq!(record.address, 0x0000);
    assert_eq!(record.data, Vec::<u8>::from([0x65, 0x66, 0x67, 0x00]));
    assert_eq!(record.checksum, 0xC6);

    let record = parse_record("S107123401020304A8").unwrap();
    assert_eq!(record.record_type, RecordType::S1);
    assert_eq!(record.byte_count, 0x07);
    assert_eq!(record.address, 0x1234);
    assert_eq!(record.data, Vec::<u8>::from([0x01, 0x02, 0x03, 0x04]));
    assert_eq!(record.checksum, 0xA8);

    let record = parse_record("S2081234560102030451").unwrap();
    assert_eq!(record.record_type, RecordType::S2);
    assert_eq!(record.byte_count, 0x08);
    assert_eq!(record.address, 0x123456);
    assert_eq!(record.data, Vec::<u8>::from([0x01, 0x02, 0x03, 0x04]));
    assert_eq!(record.checksum, 0x51);

    let record = parse_record("S3091234567801020304D8").unwrap();
    assert_eq!(record.record_type, RecordType::S3);
    assert_eq!(record.byte_count, 0x09);
    assert_eq!(record.address, 0x12345678);
    assert_eq!(record.data, Vec::<u8>::from([0x01, 0x02, 0x03, 0x04]));
    assert_eq!(record.checksum, 0xD8);

    let record = parse_record("S5031234B6").unwrap();
    assert_eq!(record.record_type, RecordType::S5);
    assert_eq!(record.byte_count, 0x03);
    assert_eq!(record.address, 0x1234);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0xB6);

    let record = parse_record("S6041234565F").unwrap();
    assert_eq!(record.record_type, RecordType::S6);
    assert_eq!(record.byte_count, 0x04);
    assert_eq!(record.address, 0x123456);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0x5F);

    let record = parse_record("S70512345678E6").unwrap();
    assert_eq!(record.record_type, RecordType::S7);
    assert_eq!(record.byte_count, 0x05);
    assert_eq!(record.address, 0x12345678);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0xE6);

    let record = parse_record("S8041234565F").unwrap();
    assert_eq!(record.record_type, RecordType::S8);
    assert_eq!(record.byte_count, 0x04);
    assert_eq!(record.address, 0x123456);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0x5F);

    let record = parse_record("S9031234B6").unwrap();
    assert_eq!(record.record_type, RecordType::S9);
    assert_eq!(record.byte_count, 0x03);
    assert_eq!(record.address, 0x1234);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0xB6);

    // Test Wikipedia example

    let record = parse_record("S00F000068656C6C6F202020202000003C").unwrap();
    assert_eq!(record.record_type, RecordType::S0);
    assert_eq!(record.byte_count, 0x0F);
    assert_eq!(record.address, 0x0000);
    assert_eq!(
        record.data,
        Vec::<u8>::from([0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00])
    );
    assert_eq!(record.checksum, 0x3C);

    let record =
        parse_record("S11F00007C0802A6900100049421FFF07C6C1B787C8C23783C6000003863000026").unwrap();
    assert_eq!(record.record_type, RecordType::S1);
    assert_eq!(record.byte_count, 0x1F);
    assert_eq!(record.address, 0x0000);
    assert_eq!(
        record.data,
        Vec::<u8>::from([
            0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF, 0xF0, 0x7C, 0x6C,
            0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63, 0x00, 0x00
        ])
    );
    assert_eq!(record.checksum, 0x26);

    let record =
        parse_record("S11F001C4BFFFFE5398000007D83637880010014382100107C0803A64E800020E9").unwrap();
    assert_eq!(record.record_type, RecordType::S1);
    assert_eq!(record.byte_count, 0x1F);
    assert_eq!(record.address, 0x001C);
    assert_eq!(
        record.data,
        Vec::<u8>::from([
            0x4B, 0xFF, 0xFF, 0xE5, 0x39, 0x80, 0x00, 0x00, 0x7D, 0x83, 0x63, 0x78, 0x80, 0x01,
            0x00, 0x14, 0x38, 0x21, 0x00, 0x10, 0x7C, 0x08, 0x03, 0xA6, 0x4E, 0x80, 0x00, 0x20
        ])
    );
    assert_eq!(record.checksum, 0xE9);

    let record = parse_record("S111003848656C6C6F20776F726C642E0A0042").unwrap();
    assert_eq!(record.record_type, RecordType::S1);
    assert_eq!(record.byte_count, 0x11);
    assert_eq!(record.address, 0x0038);
    assert_eq!(
        record.data,
        Vec::<u8>::from([
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x2E, 0x0A, 0x00
        ])
    );
    assert_eq!(record.checksum, 0x42);

    let record = parse_record("S5030003F9").unwrap();
    assert_eq!(record.record_type, RecordType::S5);
    assert_eq!(record.byte_count, 0x03);
    assert_eq!(record.address, 0x0003);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0xF9);

    let record = parse_record("S9030000FC").unwrap();
    assert_eq!(record.record_type, RecordType::S9);
    assert_eq!(record.byte_count, 0x03);
    assert_eq!(record.address, 0x0000);
    assert_eq!(record.data, Vec::<u8>::from([]));
    assert_eq!(record.checksum, 0xFC);

    // Errors
    // TODO: Compare to error types

    // Empty string
    assert_eq!(
        parse_record(""),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingRecordType
        })
    );
    // Invalid first character
    assert_eq!(
        parse_record("0"),
        Err(SRecordParseError {
            error_type: ErrorType::InvalidFirstCharacter
        })
    );

    // No record type
    assert_eq!(
        parse_record("S"),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingRecordType
        })
    );
    // Invalid record type
    assert_eq!(
        parse_record("SA"),
        Err(SRecordParseError {
            error_type: ErrorType::InvalidRecordType
        })
    );
    assert_eq!(
        parse_record("S4"),
        Err(SRecordParseError {
            error_type: ErrorType::S4Reserved
        })
    );

    // No byte count
    assert_eq!(
        parse_record("S1"),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingByteCount
        })
    );
    // Invalid byte count
    assert_eq!(
        parse_record("S1FG"),
        Err(SRecordParseError {
            error_type: ErrorType::InvalidByteCount
        })
    );

    // No address
    assert_eq!(
        parse_record("S107"),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingAddress
        })
    );
    // Invalid address
    assert_eq!(
        parse_record("S107xxxx"),
        Err(SRecordParseError {
            error_type: ErrorType::InvalidAddress
        })
    );

    // No data
    assert_eq!(
        parse_record("S1070000"),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingData
        })
    );
    // Too short data
    assert_eq!(
        parse_record("S10700001234"),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingData
        })
    );
    // Invalid data
    assert_eq!(
        parse_record("S1070000xxxxxxxx"),
        Err(SRecordParseError {
            error_type: ErrorType::InvalidData
        })
    );

    // No checksum
    assert_eq!(
        parse_record("S107000001020304"),
        Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingChecksum
        })
    );
    // Invalid checksum
    assert_eq!(
        parse_record("S107000001020304xx"),
        Err(SRecordParseError {
            error_type: ErrorType::InvalidChecksum
        })
    );
    // Incorrect checksum
    assert_eq!(
        parse_record("S10700000102030400"),
        Err(SRecordParseError {
            error_type: ErrorType::CalculatedChecksumNotMatchingParsedChecksum
        })
    );

    // Too long string
    assert_eq!(
        parse_record("S107000001020304EE0"),
        Err(SRecordParseError {
            error_type: ErrorType::LineNotTerminatedAfterChecksum
        })
    );
}

#[test]
fn test_srecord_file_new() {
    let srecord_file = SRecordFile::new();
    assert_eq!(srecord_file.header_data, []);
    assert_eq!(srecord_file.data, []);
    assert_eq!(srecord_file.start_address, None);
}

#[test]
fn test_srecord_file_default() {
    let srecord_file = Option::<SRecordFile>::None.unwrap_or_default();
    assert_eq!(srecord_file.header_data, []);
    assert_eq!(srecord_file.data, []);
    assert_eq!(srecord_file.start_address, None);
}

#[test]
fn test_srecord_file_from_str() {
    let srecord_str = fs::read_to_string("tests/srec_files/wikipedia.s19").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    assert_eq!(
        srecord_file.header_data,
        Vec::<u8>::from([0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00])
    );
    assert_eq!(
        srecord_file.data,
        Vec::<(u32, Vec<u8>)>::from([(
            0x0000,
            Vec::<u8>::from([
                0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF, 0xF0, 0x7C, 0x6C,
                0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63, 0x00, 0x00,
                0x4B, 0xFF, 0xFF, 0xE5, 0x39, 0x80, 0x00, 0x00, 0x7D, 0x83, 0x63, 0x78, 0x80, 0x01,
                0x00, 0x14, 0x38, 0x21, 0x00, 0x10, 0x7C, 0x08, 0x03, 0xA6, 0x4E, 0x80, 0x00, 0x20,
                0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x2E, 0x0A, 0x00,
            ])
        )])
    );
    assert_eq!(srecord_file.start_address, Some(0));
}

#[test]
fn test_parse_srecord_unsorted_data() {
    let srecord_str = fs::read_to_string("tests/srec_files/unsorted.s28").unwrap();
    let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

    assert_eq!(srecord_file.header_data, []);
    assert_eq!(
        srecord_file.data,
        [
            (0x01, Vec::<u8>::from([0x01, 0x02, 0x03])),
            (0x05, Vec::<u8>::from([0x05]))
        ]
    );
    assert_eq!(srecord_file.start_address, Some(0x00));
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
