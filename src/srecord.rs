use core::ops::Range;

use std::fmt;
use std::num::Wrapping;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use hex;

/// Enum containing which type a [`Record`] is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordType {
    /// Header record. 16-bit address that must be 0x0000.
    S0,
    /// 16-bit address data record.
    S1,
    /// 24-bit address data record.
    S2,
    /// 32-bit address data record.
    S3,
    // S4 is reserved
    /// 16-bit record count, containing the count of records that have occurred in the SRecord file.
    /// Can only occur once in an SRecord file and must be put last, after all the data records but
    /// before the start address record. Should be used instead of [`S6`](RecordType::S6) if the
    /// record count is less than 65536 (0x10000).
    S5,
    /// 24-bit record count, containing the count of records that have occurred in the SRecord file.
    /// Can only occur once in an SRecord file and must be put last, after all the data records but
    /// before the start address record. Should be used instead of [`S5`](RecordType::S5) if the
    /// record count is equal to or greater than 65536 (0x10000), otherwise [`S5`](RecordType::S5)
    /// shall be used instead.
    S6,
    /// 32-bit address containing the start execution location. This is an optional record, but when
    /// it occurs it must be the very last record of the file.
    S7,
    /// 24-bit address containing the start execution location. This is an optional record, but when
    /// it occurs it must be the very last record of the file.
    S8,
    /// 16-bit address containing the start execution location. This is an optional record, but when
    /// it occurs it must be the very last record of the file.
    S9,
}

impl RecordType {
    /// Returns the number of address bytes a certain record type contains.
    ///
    /// # Example
    ///
    /// ```
    /// use srex::srecord::RecordType;
    ///
    /// let record_type = RecordType::S2;
    /// assert_eq!(record_type.num_address_bytes(), 3);
    /// ```
    pub fn num_address_bytes(&self) -> usize {
        match *self {
            RecordType::S0 => 2,
            RecordType::S1 => 2,
            RecordType::S2 => 3,
            RecordType::S3 => 4,
            RecordType::S5 => 2,
            RecordType::S6 => 3,
            RecordType::S7 => 4,
            RecordType::S8 => 3,
            RecordType::S9 => 2,
        }
    }
}

impl fmt::Display for RecordType {
    /// Formats the record type including a leading S.
    ///
    /// # Example
    ///
    /// ```
    /// use srex::srecord::RecordType;
    ///
    /// let record_type = RecordType::S5;
    /// assert_eq!(format!("{record_type}"), "S5");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RecordType::S0 => write!(f, "S0"),
            RecordType::S1 => write!(f, "S1"),
            RecordType::S2 => write!(f, "S2"),
            RecordType::S3 => write!(f, "S3"),
            RecordType::S5 => write!(f, "S5"),
            RecordType::S6 => write!(f, "S6"),
            RecordType::S7 => write!(f, "S7"),
            RecordType::S8 => write!(f, "S8"),
            RecordType::S9 => write!(f, "S9"),
        }
    }
}

/// Contains the information from a parsed record.
#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    /// [`RecordType`] that specifies which type a [`Record`] is.
    pub record_type: RecordType,
    /// The parsed address from the record. Is `u32` to fit all address types in it.
    pub address: u32,
    /// The parsed data from the record. Is empty if the [`RecordType`] does not contain any data.
    pub data: Vec<u8>,
}

// TODO: pub?
#[derive(Debug, PartialEq, Eq)]
pub struct DataChunk {
    /// Inclusive address of the start of the data chunk.
    pub address: u32,
    /// Raw data of data chunk.
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct SRecordFile {
    /// Byte vector with data in header (S0).
    pub header_data: Option<Vec<u8>>,
    /// Byte vector with actual file data (S1/S2/S3).
    pub data: Vec<DataChunk>,
    /// Start address at the end of the file.
    pub start_address: Option<u32>,
}

impl Default for SRecordFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SRecordParseError {
    pub error_type: ErrorType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    /// Early, unexpected end of line when parsing record type (S*)
    EolWhileParsingRecordType,
    /// Early, unexpected end of line when parsing byte count
    EolWhileParsingByteCount,
    /// Early, unexpected end of line when parsing address
    EolWhileParsingAddress,
    /// Early, unexpected end of line when parsing data
    EolWhileParsingData,
    /// Early, unexpected end of line when parsing checksum
    EolWhileParsingChecksum,

    /// Line not terminated after checksum is parsed (supposed to be final byte of line
    LineNotTerminatedAfterChecksum,

    /// First character in record/line is not 'S'
    InvalidFirstCharacter,
    /// S4 record is reserved
    S4Reserved,
    /// Invalid character after 'S', e.g. 'SA'
    InvalidRecordType,

    /// Invalid byte count (e.g. invalid characters)
    InvalidByteCount,
    /// Byte count is too low for the minimum amount of bytes for record type
    ByteCountTooLowForRecordType,

    /// Invalid address (e.g. invalid characters)
    InvalidAddress,

    /// Invalid data (e.g. invalid characters)
    InvalidData,
    /// Overlapping data (data for same address encountered multiple times
    OverlappingData,

    /// Invalid checksum (e.g. invalid characters)
    InvalidChecksum,
    /// Calculated checksum from byte count, address and data does not match checksum parsed from
    /// file
    CalculatedChecksumNotMatchingParsedChecksum,

    /// Calculated/encountered number of records do not match what is configured in file
    CalculatedNumRecordsNotMatchingParsedNumRecords,

    /// Multiple header records (S0) found
    MultipleHeaderRecords,
    /// Multiple start addresses (S7|8|9) found
    MultipleStartAddresses,
}

impl SRecordFile {
    /// Creates a new [`SRecordFile`] object with empty `data` and `None`
    /// `header_data` and `start_address`.
    pub fn new() -> Self {
        SRecordFile {
            header_data: None,
            data: Vec::<DataChunk>::new(),
            start_address: None,
        }
    }

    /// Sorts data address ascending, and merges adjacent data together
    pub fn sort_data(&mut self) {
        self.data.sort_by(|a, b| a.address.cmp(&b.address));
        let mut new_data = Vec::<DataChunk>::new();
        for data_chunk in self.data.iter() {
            match new_data.last_mut() {
                Some(c) => {
                    if data_chunk.address as u64 == c.address as u64 + c.data.len() as u64 {
                        c.data.extend(data_chunk.data.iter());
                    } else {
                        new_data.push(DataChunk {
                            address: data_chunk.address,
                            data: data_chunk.data.to_vec(),
                        });
                    }
                }
                None => {
                    new_data.push(DataChunk {
                        address: data_chunk.address,
                        data: data_chunk.data.to_vec(),
                    });
                }
            }
        }
        self.data = new_data;
    }

    fn get_data_chunk_containing_address(&self, address: u32) -> Option<(usize, &DataChunk)> {
        let address = address as u64;
        for (i, data_chunk) in self.data.iter().enumerate() {
            let start_address = data_chunk.address as u64;
            let end_address = start_address + data_chunk.data.len() as u64;
            if start_address <= address && address < end_address {
                return Some((i, &data_chunk));
            }
        }
        None
    }
}

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
                            // TODO: Error if multiple header records instead of overwriting
                            srecord_file.header_data = Some(record.data);
                        }
                        RecordType::S1 | RecordType::S2 | RecordType::S3 => {
                            // TODO: Validate record type (no mixes?)
                            let mut is_added_to_existing_data_part = false;
                            for data_chunk in srecord_file.data.iter_mut().rev() {
                                // Appending to existing data
                                if record.address
                                    == (data_chunk.address + data_chunk.data.len() as u32)
                                {
                                    data_chunk.data.extend(&record.data);
                                    is_added_to_existing_data_part = true;
                                    break;
                                }
                            }
                            if !is_added_to_existing_data_part {
                                srecord_file.data.push(DataChunk {
                                    address: record.address,
                                    data: record.data,
                                });
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
                                return Err(SRecordParseError {
                                    error_type:
                                        ErrorType::CalculatedNumRecordsNotMatchingParsedNumRecords,
                                });
                            }
                        }
                        RecordType::S7 | RecordType::S8 | RecordType::S9 => {
                            if srecord_file.start_address.is_some() {
                                return Err(SRecordParseError {
                                    error_type: ErrorType::MultipleStartAddresses,
                                });
                            }
                            srecord_file.start_address = Some(record.address);
                        }
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        srecord_file.sort_data();

        Ok(srecord_file)
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
    /// [`index`](SRecordFile::index) will [`panic!`] if the input address does not exist in the
    /// [`SRecordFile`].
    fn index(&self, address: u32) -> &Self::Output {
        match self.get_data_chunk_containing_address(address) {
            Some((_, data_chunk)) => &data_chunk.data[(address - data_chunk.address) as usize],
            None => {
                panic!("Address {address:#04X} does not exist in SRecordFile");
            }
        }
    }
}

impl Index<Range<u32>> for SRecordFile {
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
    /// [`index`](SRecordFile::index) will [`panic!`] if the input address range does not exist in
    /// the [`SRecordFile`].
    fn index(&self, address_range: Range<u32>) -> &Self::Output {
        match self.get_data_chunk_containing_address(address_range.start) {
            Some((_, data_chunk)) => {
                let start_index = address_range.start as u64 - data_chunk.address as u64;
                let end_index = address_range.end as u64 - data_chunk.address as u64;
                match data_chunk
                    .data
                    .get(start_index as usize..end_index as usize)
                {
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
                panic!(
                    "Address range {start_address}..{end_address} does not exist in SRecordFile"
                );
            }
        }
    }
}

impl IndexMut<u32> for SRecordFile {
    /// Performs mutable indexing in [`SRecordFile`], allowing writing using syntax
    /// `srecord_file[0x1234] = 0xFF`.
    ///
    /// # Example
    ///
    /// ```
    /// use srex::srecord::SRecordFile;
    ///
    /// let mut srecord_file: SRecordFile = [
    ///     "S0070000484452001A",
    ///     "S107123401020304A8",
    ///     "S5030001FB",
    ///     "S9031234B6",
    /// ].join("\n")
    ///     .parse()
    ///     .unwrap();
    ///
    /// assert_eq!(srecord_file[0x1234], 0x01);
    /// srecord_file[0x1234] = 0xFF;
    /// assert_eq!(srecord_file[0x1234], 0xFF);
    /// ```
    ///
    /// # Panics
    ///
    /// [`index_mut`](SRecordFile::index_mut) will [`panic!`] if the input address does not exist in
    /// the [`SRecordFile`].
    ///
    /// TODO: Implement allocating data if address does not already exist in file.
    fn index_mut(&mut self, address: u32) -> &mut Self::Output {
        let address = address as u64;
        for data_chunk in self.data.iter_mut() {
            let start_address = data_chunk.address as u64;
            let end_address = start_address + data_chunk.data.len() as u64;
            if (start_address <= address) && (address < end_address) {
                return &mut data_chunk.data[(address - start_address) as usize];
            }
        }
        panic!("Address {address:#02X} does not exist in SRecordFile");
    }
}

/// Parses a record type from `record_str` and returns it, or error message
#[inline]
fn parse_record_type(record_str: &str) -> Result<RecordType, SRecordParseError> {
    let mut chars = record_str.chars();
    match chars.next() {
        Some('S') => match chars.next() {
            Some('0') => Ok(RecordType::S0),
            Some('1') => Ok(RecordType::S1),
            Some('2') => Ok(RecordType::S2),
            Some('3') => Ok(RecordType::S3),
            Some('4') => Err(SRecordParseError {
                error_type: ErrorType::S4Reserved,
            }),
            Some('5') => Ok(RecordType::S5),
            Some('6') => Ok(RecordType::S6),
            Some('7') => Ok(RecordType::S7),
            Some('8') => Ok(RecordType::S8),
            Some('9') => Ok(RecordType::S9),
            Some(_) => Err(SRecordParseError {
                error_type: ErrorType::InvalidRecordType,
            }),
            None => Err(SRecordParseError {
                error_type: ErrorType::EolWhileParsingRecordType,
            }),
        },
        Some(_) => Err(SRecordParseError {
            error_type: ErrorType::InvalidFirstCharacter,
        }),
        None => Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingRecordType,
        }),
    }
}

/// Parses byte count from `record_str` and returns it, or error message
#[inline]
fn parse_byte_count(record_str: &str) -> Result<u8, SRecordParseError> {
    match record_str.get(2..4) {
        Some(byte_count_str) => match u8::from_str_radix(byte_count_str, 16) {
            Ok(i) => Ok(i),
            Err(_) => Err(SRecordParseError {
                error_type: ErrorType::InvalidByteCount,
            }),
        },
        None => {
            return Err(SRecordParseError {
                error_type: ErrorType::EolWhileParsingByteCount,
            })
        }
    }
}

/// Parses address from `record_str` and returns it, or error message
#[inline]
fn parse_address(record_str: &str, record_type: RecordType) -> Result<u32, SRecordParseError> {
    let num_address_bytes = record_type.num_address_bytes();
    let num_address_chars = num_address_bytes * 2;
    let address_start_index = 4;
    let address_end_index = address_start_index + num_address_chars;

    match record_str.get(address_start_index..address_end_index) {
        Some(address_str) => match u32::from_str_radix(address_str, 16) {
            Ok(i) => Ok(i),
            Err(_) => Err(SRecordParseError {
                error_type: ErrorType::InvalidAddress,
            }),
        },
        None => Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingAddress,
        }),
    }
}

/// Parses data and sets slice inside record
///
/// Data is added to `self` and a slice to the data is set to `record`
#[inline]
fn parse_data_and_checksum(
    record_str: &str,
    record_type: RecordType,
    byte_count: u8,
    address: u32,
) -> Result<Vec<u8>, SRecordParseError> {
    // TODO: Validate record type?

    let num_address_bytes = record_type.num_address_bytes();
    let num_data_bytes = match (byte_count as usize).checked_sub(num_address_bytes + 1) {
        Some(i) => i,
        None => {
            return Err(SRecordParseError {
                error_type: ErrorType::ByteCountTooLowForRecordType,
            })
        }
    };

    // Parse data
    let data_start_index = 2 + 2 + 2 * num_address_bytes; // S* + byte count + address
    let data_end_index = data_start_index + num_data_bytes * 2;
    let record_data = match record_str.get(data_start_index..data_end_index) {
        Some(data_str) => match hex::decode(data_str) {
            Ok(vec) => vec,
            Err(_) => {
                return Err(SRecordParseError {
                    error_type: ErrorType::InvalidData,
                })
            }
        },
        None => {
            return Err(SRecordParseError {
                error_type: ErrorType::EolWhileParsingData,
            })
        }
    };

    // Next, parse and validate checksum
    let checksum_start_index = data_end_index;
    let checksum_end_index = checksum_start_index + 2;
    let checksum: u8 = match record_str.get(checksum_start_index..checksum_end_index) {
        Some(checksum_str) => match u8::from_str_radix(checksum_str, 16) {
            Ok(i) => i,
            Err(_) => {
                return Err(SRecordParseError {
                    error_type: ErrorType::InvalidChecksum,
                });
            }
        },
        None => {
            return Err(SRecordParseError {
                error_type: ErrorType::EolWhileParsingChecksum,
            });
        }
    };
    let expected_checksum = calculate_checksum(byte_count, address, record_data.as_slice());
    if checksum != expected_checksum {
        return Err(SRecordParseError {
            error_type: ErrorType::CalculatedChecksumNotMatchingParsedChecksum,
        });
    }

    // Finally, validate that we are at the end of the record str
    if record_str.len() != checksum_end_index {
        return Err(SRecordParseError {
            error_type: ErrorType::LineNotTerminatedAfterChecksum,
        });
    }

    Ok(record_data)
}

/// Calculate the checksum for a single record (line).
///
/// The checksum is calculated from the sum of all the individual bytes, from `byte_count`,
/// individual `address` bytes and all bytes in `data`. All but the least significant byte is
/// discarded, and is then bitwise inverted.
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
pub fn parse_record(record_str: &str) -> Result<Record, SRecordParseError> {
    let record_type = parse_record_type(record_str)?;
    let byte_count = parse_byte_count(record_str)?;
    let address = parse_address(record_str, record_type.clone())?;
    let data = parse_data_and_checksum(record_str, record_type.clone(), byte_count, address)?;
    Ok(Record {
        record_type,
        address,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_type_fmt() {
        let s0 = RecordType::S0;
        let s1 = RecordType::S1;
        let s2 = RecordType::S2;
        let s3 = RecordType::S3;
        let s5 = RecordType::S5;
        let s6 = RecordType::S6;
        let s7 = RecordType::S7;
        let s8 = RecordType::S8;
        let s9 = RecordType::S9;
        assert_eq!(format!("{s0}"), "S0");
        assert_eq!(format!("{s1}"), "S1");
        assert_eq!(format!("{s2}"), "S2");
        assert_eq!(format!("{s3}"), "S3");
        assert_eq!(format!("{s5}"), "S5");
        assert_eq!(format!("{s6}"), "S6");
        assert_eq!(format!("{s7}"), "S7");
        assert_eq!(format!("{s8}"), "S8");
        assert_eq!(format!("{s9}"), "S9");
    }

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
                    0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF, 0xF0, 0x7C,
                    0x6C, 0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63,
                    0x00, 0x00
                ]
            ),
            0x26
        );
        assert_eq!(
            calculate_checksum(
                0x1F,
                0x001C,
                &[
                    0x4B, 0xFF, 0xFF, 0xE5, 0x39, 0x80, 0x00, 0x00, 0x7D, 0x83, 0x63, 0x78, 0x80,
                    0x01, 0x00, 0x14, 0x38, 0x21, 0x00, 0x10, 0x7C, 0x08, 0x03, 0xA6, 0x4E, 0x80,
                    0x00, 0x20
                ]
            ),
            0xE9
        );
        assert_eq!(
            calculate_checksum(
                0x11,
                0x0038,
                &[
                    0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x2E, 0x0A,
                    0x00
                ]
            ),
            0x42
        );
        assert_eq!(calculate_checksum(0x03, 0x0003, &[]), 0xF9);
        assert_eq!(calculate_checksum(0x03, 0x0000, &[]), 0xFC);
    }
}
