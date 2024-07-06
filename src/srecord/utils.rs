use std::num::Wrapping;

use crate::srecord::error::{ErrorType, SRecordParseError};
use crate::srecord::record_type::RecordType;

/// Parses a record type from `record_str` and returns it, or error message
#[inline]
pub(crate) fn parse_record_type(record_str: &str) -> Result<RecordType, SRecordParseError> {
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
pub(crate) fn parse_byte_count(record_str: &str) -> Result<u8, SRecordParseError> {
    match record_str.get(2..4) {
        Some(byte_count_str) => match u8::from_str_radix(byte_count_str, 16) {
            Ok(i) => Ok(i),
            Err(_) => Err(SRecordParseError {
                error_type: ErrorType::InvalidByteCount,
            }),
        },
        None => Err(SRecordParseError {
            error_type: ErrorType::EolWhileParsingByteCount,
        }),
    }
}

/// Parses address from `record_str` and returns it, or error message
#[inline]
pub(crate) fn parse_address(
    record_str: &str,
    record_type: &RecordType,
) -> Result<u64, SRecordParseError> {
    let num_address_bytes = record_type.num_address_bytes();
    let num_address_chars = num_address_bytes * 2;
    let address_start_index = 4;
    let address_end_index = address_start_index + num_address_chars;

    match record_str.get(address_start_index..address_end_index) {
        Some(address_str) => match u64::from_str_radix(address_str, 16) {
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
/// Data is written to `data`.
#[inline]
pub(crate) fn parse_data_and_checksum(
    record_str: &str,
    record_type: RecordType,
    byte_count: u8,
    address: u64,
    data: &mut [u8],
) -> Result<(), SRecordParseError> {
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
    let data = &mut data[..num_data_bytes];

    // Parse data
    let data_start_index = 2 + 2 + 2 * num_address_bytes; // S* + byte count + address
    let data_end_index = data_start_index + num_data_bytes * 2;
    match record_str.get(data_start_index..data_end_index) {
        Some(data_str) => match hex::decode_to_slice(data_str, data) {
            Ok(_) => {}
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
    let expected_checksum = calculate_checksum(byte_count, address, data);
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

    Ok(())
}

/// Calculate the checksum for a single record (line).
///
/// The checksum is calculated from the sum of all the individual bytes, from `byte_count`,
/// individual `address` bytes and all bytes in `data`. All but the least significant byte is
/// discarded, and is then bitwise inverted.
///
/// TODO: Pub????
pub fn calculate_checksum(byte_count: u8, address: u64, data: &[u8]) -> u8 {
    let mut checksum = Wrapping(byte_count);
    for byte in address.to_be_bytes().iter() {
        checksum += byte;
    }
    for byte in data.iter() {
        checksum += byte;
    }
    0xFF - checksum.0
}

#[cfg(test)]
mod tests {
    use super::*;

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
