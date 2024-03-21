use std::num::Wrapping;

use hex::decode;

#[derive(Debug, PartialEq, Eq)]
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
    pub record_type: RecordType,
    pub byte_count: u8,
    pub address: u32,
    pub data: Vec<u8>, // TODO: array slice?
    pub checksum: u8,
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
///
/// TODO: Make string subslicing safe and fast
///
/// TODO: Add support for endianness
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
        Some(c) => { record_type_char = c; }
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
        c => { return Result::Err(format!("Failed to parse '{record_str}': invalid record type S{c}")); }
    }

    // Next, parse byte-count
    let byte_count: u8;
    let byte_count_str = &record_str[2..4];
    match u8::from_str_radix(byte_count_str, 16) {
        Ok(i) => { byte_count = i; }
        Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse byte count from '{byte_count_str}'")); }
    }

    // Next, parse address
    let num_address_bytes = match record_type {
        RecordType::S0 => 2,
        RecordType::S1 => 2,
        RecordType::S2 => 3,
        RecordType::S3 => 4,
        RecordType::S5 => 2,
        RecordType::S6 => 3,
        RecordType::S7 => 4,
        RecordType::S8 => 3,
        RecordType::S9 => 2,
    };
    let num_address_chars = num_address_bytes * 2;
    let address_str = &record_str[4..4+num_address_chars];
    let address: u32;
    match u32::from_str_radix(address_str, 16) {
        Ok(i) => { address = i; }
        Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse address from '{address_str}'")); }
    }

    // Next, parse data
    let data_start_index = 4 + num_address_chars;
    let data_end_index = data_start_index + 2 * ((byte_count as usize) - num_address_bytes - 1);
    let data_str = &record_str[data_start_index..data_end_index];
    let data: Vec<u8>;
    match hex::decode(data_str) {
        Ok(vec) => { data = vec; }
        Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse data from '{data_str}'")); }
    }

    // Next, parse and validate checksum
    let checksum_start_index = data_end_index;
    let checksum_end_index = checksum_start_index + 2;
    let checksum_str = &record_str[checksum_start_index..checksum_end_index];
    let checksum: u8;
    match u8::from_str_radix(checksum_str, 16) {
        Ok(i) => { checksum = i; }
        Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse checksum from '{checksum_str}'")); }
    }
    let expected_checksum = calculate_checksum(byte_count, address, &data);
    if checksum != expected_checksum {
        return Result::Err(format!("Failed to parse '{record_str}': calculated checksum {expected_checksum:#02X} does not match parsed checksum {checksum:#02X}"));
    }
    // TODO: Validate checksum

    // Finally, validate that we are at the end of the record str
    if record_str.len() != checksum_end_index {
        let remaining_str = &record_str[checksum_end_index..];
        return Result::Err(format!("Failed to parse '{record_str}': expected {checksum_end_index} characters but the string slice continued with '{remaining_str}'"));
    }

    Result::Ok(Record {
        record_type: record_type,
        byte_count: byte_count,
        address: address,
        data: data,
        checksum: checksum,
    })
}