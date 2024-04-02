use core::ops::Range;

use std::num::Wrapping;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::str::FromStr;

use hex;

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

#[derive(Debug)]
// TODO: Implement slicing to get data at address (or something)
pub struct SRecordFile {
    pub file_path: Option<PathBuf>,
    pub header_data: Vec<u8>,
    pub data: Vec<(u32, Vec<u8>)>,
    pub start_address: Option<u32>,
}

impl Default for SRecordFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SRecordParseError(String);

impl FromStr for SRecordFile {
    type Err = SRecordParseError;

    fn from_str(srecord_str: &str) -> Result<Self, Self::Err> {
        let mut srecord_file = SRecordFile::new();

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
                            let file_num_records = record.address;
                            if num_data_records != file_num_records {
                                return Result::Err(SRecordParseError(format!("Failed to parse SRecord: Number of records found in file ({num_data_records:#02X}) does not match record count in file ({line} - {file_num_records:#02X}")));
                            }
                        }
                        RecordType::S7 | RecordType::S8 | RecordType::S9 => {
                            if srecord_file.start_address.is_some() {
                                return Result::Err(SRecordParseError(String::from("Failed to parse SRecord: multiple start address records found")));
                            }
                            srecord_file.start_address = Some(record.address);
                        }
                    }
                }
                Err(msg) => { return Result::Err(SRecordParseError(msg)); }
            }
        }

        srecord_file.sort_data();

        Ok(srecord_file)
    }
}

impl SRecordFile {
    pub fn new() -> Self {
        SRecordFile {
            file_path: None,
            header_data: Vec::<u8>::new(),
            data: Vec::<(u32, Vec<u8>)>::new(),
            start_address: None,
        }
    }

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

    fn get_vec_containing_address(&self, address: u32) -> Option<(usize, u32, &Vec<u8>)> {
        let address = address as u64;
        for (i, (start_address, vec)) in self.data.iter().enumerate() {
            let start_address = *start_address as u64;
            let end_address = start_address + vec.len() as u64;
            if start_address <= address && address < end_address {
                return Some((i, start_address as u32, vec));
            }
        }
        None
    }
}

impl Index<u32> for SRecordFile {
    type Output = u8;

    /// Index the data inside the [`SRecordFile`] using the syntax
    /// `srecord_file[0x1234]`, where `0x1234` is the address inside the SRecord file.
    ///
    /// # Examples
    ///
    /// ```
    /// use srex::srecord::SRecordFile;
    ///
    /// let srecord_file: SRecordFile = [
    ///     "S0070000484452001A",
    ///     "S104123401B4",
    ///     "S5030001FB",
    ///     "S9031234B6",
    /// ].join("\n")
    ///     .parse()
    ///     .unwrap();
    ///
    /// // This will panic if 0x1234 does not exist in srecord_file
    /// let value: u8 = srecord_file[0x1234];
    /// println!("value = {value}");
    /// ```
    ///
    /// # Panics
    ///
    /// [`index`](SRecordFile::index) will [`panic!`] if the input address does not exist in the SRecord file.
    fn index(&self, address: u32) -> &Self::Output {
        match self.get_vec_containing_address(address) {
            Some((_, start_address, vec)) => {
                &vec[(address - start_address) as usize]
            }
            None => {
                panic!("Address {address:#04X} does not exist in SRecordFile");
            }
        }
    }
}

impl Index<Range<u32>> for SRecordFile
{
    type Output = [u8];

    /// Get a slice for the data inside the [`SRecordFile`] using the syntax
    /// `srecord_file[0x1235..0x1237]`, where `0x1235` and `0x1237` are addresses inside the SRecord
    /// file.
    ///
    /// # Examples
    ///
    /// ```
    /// use srex::srecord::SRecordFile;
    ///
    /// let srecord_file: SRecordFile = [
    ///     "S0070000484452001A",
    ///     "S107123401020304A8",
    ///     "S5030001FB",
    ///     "S9031234B6",
    /// ].join("\n")
    ///     .parse()
    ///     .unwrap();
    ///
    /// // This will panic if 0x1235..0x1237 does not exist in srecord_file
    /// let slice: &[u8] = &srecord_file[0x1235..0x1237];
    /// let x: u8 = slice[0];
    /// let y: u8 = slice[1];
    /// println!("x = {x}, y = {y}");
    /// ```
    ///
    /// # Panics
    ///
    /// [`index`](SRecordFile::index) will [`panic!`] if the input address does not exist in the SRecord file.
    fn index(&self, address_range: Range<u32>) -> &Self::Output {
        match self.get_vec_containing_address(address_range.start) {
            Some((_, start_address, data)) => {
                let start_index = address_range.start as u64 - start_address as u64;
                let end_index = address_range.end as u64 - start_address as u64;
                match data.get(start_index as usize .. end_index as usize) {
                    Some(slice) => slice,
                    None => {
                        let start_address = address_range.start;
                        let end_address = address_range.end;
                        panic!("Address range {start_address}..{end_address} is not fully contained in SRecordFile");
                    }
                }
            }
            None => {
                let start_address = address_range.start;
                let end_address = address_range.end;
                panic!("Address range {start_address}..{end_address} does not exist in SRecordFile");
            }
        }
    }
}

impl IndexMut<u32> for SRecordFile {
    fn index_mut(&mut self, address: u32) -> &mut Self::Output {
        let address = address as u64;
        for (start_address, data) in self.data.iter_mut() {
            let start_address = *start_address as u64;
            let end_address = start_address + data.len() as u64;
            if (start_address <= address) && (address < end_address) {
                return &mut data[(address - start_address) as usize];
            }
        }
        panic!("Address {address:#02X} does not exist in SRecordFile");
    }
}

/// Calculate the checksum for a single record (line).
///
/// The checksum is calculated from the sum of all the individual bytes, from `byte_count`,
/// individual `address` bytes and all bytes in `data`. All but the least significant byte is
/// discarded, and is then bitwise inverted.
///
/// # Example
///
/// ```
/// use srex::srecord::calculate_checksum;
/// let byte_count: u8 = 0x07;
/// let address: u32 = 0x1234;
/// let data: &[u8] = &[0x01, 0x02, 0x03, 0x04];
///
/// assert_eq!(calculate_checksum(byte_count, address, data), 0xA8);
/// ```
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
    let record_type_char = match record_str.chars().nth(1) {
        Some(c) => c,
        None => { return Result::Err(format!("Failed to parse '{record_str}': unexpected end of string when parsing record type")); },
    };
    let record_type = match record_type_char {
        '0' => RecordType::S0,
        '1' => RecordType::S1,
        '2' => RecordType::S2,
        '3' => RecordType::S3,
        '4' => { return Result::Err(format!("Failed to parse '{record_str}': record type S4 is reserved")); }
        '5' => RecordType::S5,
        '6' => RecordType::S6,
        '7' => RecordType::S7,
        '8' => RecordType::S8,
        '9' => RecordType::S9,
        c => { return Result::Err(format!("Failed to parse '{record_str}': invalid record type S{c}")); }
    };

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
        record_type,
        byte_count,
        address,
        data,
        checksum,
    })
}
