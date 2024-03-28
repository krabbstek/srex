use std::num::Wrapping;
use std::path::PathBuf;

use hex;
use log::*;

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

// TODO: Implement slicing to get data at address (or something)
pub struct SRecordFile {
    pub file_path: Option<PathBuf>,
    pub header_data: Vec<u8>,
    pub data: Vec<(u32, Vec<u8>)>,
    pub start_address: Option<u32>,
}

impl SRecordFile {
    /// Sorts data address ascending, and merges adjacent data together
    pub fn sort_data(&mut self) {
        self.data.sort_by(|a, b| a.0.cmp(&b.0));
        let mut new_data = Vec::<(u32, Vec<u8>)>::new();
        for (address, vec) in self.data.iter() {
            match new_data.last_mut() {
                Some(c) => {
                    if *address as u64 == c.0 as u64 + c.1.len() as u64 {
                        c.1.extend(vec);
                    } else {
                        new_data.push((*address, vec.to_vec()));
                    }
                }
                None => { new_data.push((*address, vec.to_vec())); }
            }
        }
        self.data = new_data;
    }
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
    match record_str.get(2..4) {
        None => { return Result::Err(format!("Failed to parse '{record_str}': unexpected end of string when parsing byte count")); }
        Some(byte_count_str) => {
            match u8::from_str_radix(byte_count_str, 16) {
                Ok(i) => { byte_count = i; }
                Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse byte count from '{byte_count_str}'")); }
            }
        }
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
    let address: u32;
    match record_str.get(4..4+num_address_chars) {
        None => { return Result::Err(format!("Failed to parse '{record_str}': unexpected end of string when parsing address")); }
        Some(address_str) => {
            match u32::from_str_radix(address_str, 16) {
                Ok(i) => { address = i; }
                Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse address from '{address_str}'")); }
            }
        }
    }

    // Next, parse data
    println!("{record_str}");
    let data_start_index = 4 + num_address_chars;
    let data_end_index = data_start_index + 2 * (byte_count as usize) - 2 * (num_address_bytes + 1);
    let data: Vec<u8>;
    match record_str.get(data_start_index..data_end_index) {
        None => { return Result::Err(format!("Failed to parse '{record_str}': unexpected end of string when parsing data")); }
        Some(data_str) => {
            match hex::decode(data_str) {
                Ok(vec) => { data = vec; }
                Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse data from '{data_str}'")); }
            }
        }
    }

    // Next, parse and validate checksum
    let checksum_start_index = data_end_index;
    let checksum_end_index = checksum_start_index + 2;
    let checksum: u8;
    match record_str.get(checksum_start_index..checksum_end_index) {
        None => { return Result::Err(format!("Failed to parse '{record_str}': unexpected end of string when parsing checksum")); }
        Some(checksum_str) => {
            match u8::from_str_radix(checksum_str, 16) {
                Ok(i) => { checksum = i; }
                Err(_) => { return Result::Err(format!("Failed to parse '{record_str}': could not parse checksum from '{checksum_str}'")); }
            }
        }
    }
    let expected_checksum = calculate_checksum(byte_count, address, &data);
    if checksum != expected_checksum {
        return Result::Err(format!("Failed to parse '{record_str}': calculated checksum {expected_checksum:#02X} does not match parsed checksum {checksum:#02X}"));
    }

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

pub fn parse_srecord_str(srecord_str: &str) -> SRecordFile {
    let mut srecord_file = SRecordFile {
        file_path: None,
        header_data: Vec::<u8>::new(),
        data: Vec::<(u32, Vec<u8>)>::new(),
        start_address: None,
    };

    let mut num_data_records: u32 = 0;

    for line in srecord_str.lines() {
        match parse_record(line) {
            Ok(record) => {
                match record.record_type {
                    RecordType::S0 => {
                        srecord_file.header_data.extend(&record.data);
                    }
                    RecordType::S1 | RecordType::S2 | RecordType::S3 => {
                        // TODO: Validate record type (no mixes?)
                        let mut is_added_to_existing_data_part = false;
                        for data_part in srecord_file.data.iter_mut().rev() { // TODO: Rename data_part
                            // Appending to existing data
                            if record.address == (data_part.0 + data_part.1.len() as u32) {
                                data_part.1.extend(&record.data);
                                is_added_to_existing_data_part = true;
                                break;
                            }
                        }
                        if !is_added_to_existing_data_part {
                            srecord_file.data.push((record.address, record.data));
                        }
                        num_data_records += 1;
                    }
                    RecordType::S5 | RecordType::S6 => {
                        // TODO: Validate record count
                        // * Only last in file
                        // * Only once
                        // * Ensure it matches number of encountered data records
                    }
                    RecordType::S7 | RecordType::S8 | RecordType::S9 => {
                        if srecord_file.start_address != None {
                            warn!(target: "srex", "Multiple start addresses encountered");
                        }
                        srecord_file.start_address = Some(record.address);
                    }
                }
            }
            Err(msg) => { panic!("{msg}"); }
        }
    }

    srecord_file.sort_data();

    srecord_file
}
