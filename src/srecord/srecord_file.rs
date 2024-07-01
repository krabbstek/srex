use std::cmp::Ordering;
use std::ops::{Index, IndexMut, Range};
use std::str::FromStr;

use crate::srecord::data_chunk::DataChunk;
use crate::srecord::error::{ErrorType, SRecordParseError};
use crate::srecord::record_type::RecordType;
use crate::srecord::utils::{
    parse_address, parse_byte_count, parse_data_and_checksum, parse_record_type,
};

#[derive(Debug)]
pub struct SRecordFile {
    /// Byte vector with data in header (S0).
    pub header_data: Option<Vec<u8>>,
    /// Byte vector with actual file data (S1/S2/S3).
    pub data_chunks: Vec<DataChunk>,
    /// Start address at the end of the file.
    pub start_address: Option<u32>,
}

impl Default for SRecordFile {
    fn default() -> Self {
        Self::new()
    }
}

impl SRecordFile {
    /// Creates a new [`SRecordFile`] object with empty `data` and `None`
    /// `header_data` and `start_address`.
    pub fn new() -> Self {
        SRecordFile {
            header_data: None,
            data_chunks: Vec::<DataChunk>::new(),
            start_address: None,
        }
    }

    // TODO: Documentation
    // TODO: Unit tests
    fn get_data_chunk_index(&self, address: u32, inclusive_end: bool) -> Result<usize, usize> {
        let mut left_index = 0;
        let mut right_index = self.data_chunks.len();
        loop {
            let index_diff = right_index - left_index;
            if index_diff == 0 {
                return Err(left_index);
            }
            // TODO: u32 vs u64?
            let data_chunk = &self.data_chunks[left_index];
            let data_chunk_start_address = data_chunk.address;
            let mut data_chunk_end_address =
                data_chunk_start_address + data_chunk.data.len() as u32;
            if inclusive_end {
                data_chunk_end_address += 1;
            }
            if index_diff == 1 {
                if address >= data_chunk_start_address && address < data_chunk_end_address {
                    return Ok(left_index);
                } else if address < data_chunk_start_address {
                    return Err(left_index);
                } else {
                    return Err(right_index);
                }
            } else {
                let middle_index = self.data_chunks.len() / 2;
                if address < data_chunk_start_address {
                    right_index = middle_index;
                } else if address >= data_chunk_end_address {
                    left_index = middle_index;
                } else {
                    return Ok(left_index);
                }
            }
        }
    }

    // TODO: Documentation
    // TODO: Tests
    fn get_data_chunk(&self, address: u32) -> Option<&DataChunk> {
        match self.get_data_chunk_index(address, false) {
            Ok(data_chunk_index) => Some(&self.data_chunks[data_chunk_index]),
            Err(_) => None,
        }
    }

    // TODO: Documentation
    // TODO: Allocation???
    // TODO: Tests
    fn get_data_chunk_mut(&mut self, address: u32) -> Option<&mut DataChunk> {
        match self.get_data_chunk_index(address, false) {
            Ok(data_chunk_index) => Some(&mut self.data_chunks[data_chunk_index]),
            Err(_) => None,
        }
    }

    // TODO: Documentation
    fn merge_data_chunks(&mut self) -> Result<(), SRecordParseError> {
        let mut index = 0;
        while index < self.data_chunks.len() - 1 {
            let current_end_address =
                self.data_chunks[index].address as u64 + self.data_chunks[index].data.len() as u64;
            let next_index = index + 1;
            let next_start_address = self.data_chunks[next_index].address as u64;
            match next_start_address.cmp(&current_end_address) {
                Ordering::Greater => index += 1,
                Ordering::Equal => {
                    // Merge
                    let mut next_data_chunk = self.data_chunks.remove(next_index);
                    self.data_chunks[index]
                        .data
                        .append(&mut next_data_chunk.data);
                }
                Ordering::Less => {
                    return Err(SRecordParseError {
                        error_type: ErrorType::OverlappingData,
                    })
                }
            }
        }
        Ok(())
    }
}

impl FromStr for SRecordFile {
    type Err = SRecordParseError;

    fn from_str(srecord_str: &str) -> Result<Self, Self::Err> {
        let mut srecord_file = SRecordFile::new();

        let mut num_data_records: u32 = 0;
        let mut data_buffer = [0u8; 256];

        for line in srecord_str.lines() {
            let record_type = parse_record_type(line)?;
            let byte_count = parse_byte_count(line)?;
            let address = parse_address(line, &record_type)?;
            let num_data_bytes = record_type.num_data_bytes(byte_count as usize);
            parse_data_and_checksum(
                line,
                record_type.clone(),
                byte_count,
                address,
                &mut data_buffer,
            )?;
            let data = &data_buffer[..num_data_bytes];

            match record_type {
                RecordType::S0 => {
                    // TODO: Error if multiple header records instead of overwriting
                    srecord_file.header_data = Some(Vec::<u8>::from(data));
                }
                RecordType::S1 | RecordType::S2 | RecordType::S3 => {
                    // TODO: Validate record type (no mixes?)
                    match srecord_file.get_data_chunk_index(address, true) {
                        Ok(data_chunk_index) => {
                            // Error if writing to the same address twice
                            let data_chunk = &mut srecord_file.data_chunks[data_chunk_index];
                            if data_chunk.address as usize + data_chunk.data.len()
                                != address as usize
                            {
                                return Err(SRecordParseError {
                                    error_type: ErrorType::OverlappingData,
                                });
                            }
                            data_chunk.data.extend_from_slice(data);
                        }
                        Err(data_chunk_index) => {
                            // TODO: Move out to allocation function?
                            srecord_file.data_chunks.insert(
                                data_chunk_index,
                                DataChunk {
                                    address,
                                    data: Vec::<u8>::from(data),
                                },
                            );
                        }
                    }
                    num_data_records += 1;
                }
                RecordType::S5 | RecordType::S6 => {
                    // TODO: Validate record count
                    // * Only last in file
                    // * Only once
                    // * Ensure it matches number of encountered data records
                    let file_num_records = address;
                    if num_data_records != file_num_records {
                        return Err(SRecordParseError {
                            error_type: ErrorType::CalculatedNumRecordsNotMatchingParsedNumRecords,
                        });
                    }
                }
                RecordType::S7 | RecordType::S8 | RecordType::S9 => {
                    if srecord_file.start_address.is_some() {
                        return Err(SRecordParseError {
                            error_type: ErrorType::MultipleStartAddresses,
                        });
                    }
                    srecord_file.start_address = Some(address);
                }
            }
        }

        // Merge data chunks
        srecord_file.merge_data_chunks()?;

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
        match self.get_data_chunk(address) {
            Some(data_chunk) => &data_chunk.data[(address - data_chunk.address) as usize],
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
        match self.get_data_chunk(address_range.start) {
            Some(data_chunk) => {
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
        match self.get_data_chunk_mut(address) {
            // TODO: Direct address indexing?
            Some(data_chunk) => &mut data_chunk.data[(address - data_chunk.address) as usize],
            None => panic!("Address {address:#08X} does not exist in SRecordFile"),
        }
    }
}
