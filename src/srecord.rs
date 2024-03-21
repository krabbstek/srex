use std::num::Wrapping;

pub enum RecordType {
    S0, // Header
    S1, // 16-bit address data
    S2, // 24-bit address data
    S3, // 32-bit address data
    // S4 is reserved
    S5, // 16-bit count
    S6, // 24-bit count
    S7, // 32-bit start address
    S8, // 24-bit start address
    S9, // 16-bit start address
}

pub struct Record {
    record_type: RecordType,
    byte_count: u8,
    address: u32,
    data: Vec<u8>, // TODO: array slice?
    checksum: u8,
}

pub fn calculate_checksum(byte_count: u8, address: u32, data: &[u8]) -> u8 {
    let mut checksum = Wrapping(byte_count);
    for byte in address.to_be_bytes().iter() {
        checksum += byte;
    }
    for byte in data.iter() {
        checksum += byte;
    }
    0xFF - checksum.0
}

/// Parse a record (single line) from an SRecord file.
pub fn parse_record(record_str: &str) -> Result<Record, String> {
    // First char is supposed to be an S
    match record_str.chars().next() {
        Some('S') => {}
        Some(other_char) => {
            return Result::Err(format!("Failed to parse '{record_str}': expected first character in record to be 'S' but was {other_char}"));
        }
        None => {
            return Result::Err(String::from("Empty string slice passed into parse_record"));
        }
    }

    // Next, parse record type
    let record_type_char: char;
    match record_str.chars().nth(1) {
        Some(c) if ('0'..='9').contains(&c) => { record_type_char = c; }
        Some(c) => { return Result::Err(format!("Failed to parse '{record_str}': invalid record type '{c}'")); }
        None => { return Result::Err(format!("Failed to parse '{record_str}': unexpected end of string when parsing record type")); }
    }
    let record_type: RecordType;
    match record_type_char {
        '0' => { record_type = RecordType::S0; }
        '1' => { record_type = RecordType::S1; }
        '2' => { record_type = RecordType::S2; }
        '3' => { record_type = RecordType::S3; }
        '4' => { return Result::Err(format!("Failed to parse '{record_str}': record type S4 is reserved")); }
        '5' => { record_type = RecordType::S5; }
        '6' => { record_type = RecordType::S6; }
        '7' => { record_type = RecordType::S7; }
        '8' => { record_type = RecordType::S8; }
        '9' => { record_type = RecordType::S9; }
        // This should never happen, but is still required by rust
        _ => { panic!("Oh no, how on earth did we end up here?!"); }
    }

    // Next, parse byte-count
    let byte_count: u8;
    let byte_count_str = &record_str[2..4];
    match u8::from_str_radix(byte_count_str, 16) {
        Ok(i) => { byte_count = i; }
        Err(_) => { return Err(format!("Failed to parse '{record_str}': could not parse byte count from '{byte_count_str}'")); }
    }

    let address = 0;
    let data = Vec::<u8>::from([]);
    let checksum = calculate_checksum(byte_count, address, &data);

    Result::Ok(Record {
        record_type: record_type,
        byte_count: byte_count,
        address: address,
        data: data,
        checksum: checksum,
    })
}
