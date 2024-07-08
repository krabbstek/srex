use std::cmp::Ordering;
use std::ops::{Index, IndexMut, Range};
use std::str::FromStr;

use crate::srecord::data_chunk::{DataChunk, DataChunkIterator};
use crate::srecord::error::{ErrorType, SRecordParseError};
use crate::srecord::slice_index::SliceIndex;
use crate::srecord::{CountRecord, HeaderRecord, Record, StartAddressRecord};

/// Struct that represents an SRecord file. It only contains the raw data, not the layout of the
/// input file.
#[derive(Debug)]
pub struct SRecordFile {
    /// Byte vector with data in header (S0).
    pub header_data: Option<Vec<u8>>,
    /// Byte vector with actual file data (S1/S2/S3).
    // TODO: Make private?
    pub data_chunks: Vec<DataChunk>,
    /// Start address at the end of the file (S7/S8/S9).
    pub start_address: Option<u64>,
}

impl Default for SRecordFile {
    fn default() -> Self {
        Self::new()
    }
}

impl SRecordFile {
    /// Creates a new [`SRecordFile`] object with empty [`data_chunks`](`SRecordFile::data_chunks`)
    /// and `None` [`header_data`](`SRecordFile::header_data`) and
    /// [`start_address`](`SRecordFile::start_address`).
    pub fn new() -> Self {
        SRecordFile {
            header_data: None,
            data_chunks: Vec::<DataChunk>::new(),
            start_address: None,
        }
    }

    /// Returns a reference to a byte or byte data subslice depending on the type of index.
    ///
    /// - If given an address, returns a reference to the byte at that address or `None` if out of
    ///   bounds.
    /// - If given an address range, returns the data subslice corresponding to that range, or
    ///   `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use srex::srecord::SRecordFile;
    ///
    /// let srecord_file = SRecordFile::from_str("S107100000010203E2").unwrap();
    ///
    /// // Single addresses return single bytes of data.
    /// assert!(srecord_file.get(0).is_none());
    /// assert_eq!(*srecord_file.get(0x1000).unwrap(), 0);
    /// assert_eq!(*srecord_file.get(0x1001).unwrap(), 1);
    /// assert_eq!(*srecord_file.get(0x1002).unwrap(), 2);
    /// assert_eq!(*srecord_file.get(0x1003).unwrap(), 3);
    /// assert!(srecord_file.get(0x1004).is_none());
    ///
    /// // Address ranges return slices of data.
    /// assert_eq!(srecord_file.get(0x1000..0x1004).unwrap(), [0u8, 1u8, 2u8, 3u8]);
    /// assert!(srecord_file.get(0x1000..0x1005).is_none());
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<Self>,
    {
        index.get(self)
    }

    /// Returns a mutable reference to a byte or byte data subslice depending on the type of index.
    ///
    /// - If given an address, returns a mutable reference to the byte at that address or `None` if
    ///   out of bounds.
    /// - If given an address range, returns the data subslice corresponding to that range, or
    ///   `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use srex::srecord::SRecordFile;
    ///
    /// let mut srecord_file = SRecordFile::from_str("S10810000001020304DD").unwrap();
    ///
    /// // Single address return single bytes of data.
    /// assert_eq!(*srecord_file.get(0x1004).unwrap(), 0x04);
    /// *srecord_file.get_mut(0x1004).unwrap() = 0xAA;
    /// assert_eq!(*srecord_file.get(0x1004).unwrap(), 0xAA);
    ///
    /// // Address ranges return slices of data.
    /// assert_eq!(srecord_file.get(0x1000..0x1004).unwrap(), [0u8, 1u8, 2u8, 3u8]);
    /// for (i, b) in srecord_file.get_mut(0x1000..0x1004).unwrap().iter_mut().enumerate() {
    ///     *b = 0x10 + i as u8;
    /// }
    /// assert_eq!(srecord_file.get(0x1000..0x1004).unwrap(), [0x10u8, 0x11u8, 0x12u8, 0x13u8]);
    /// ```
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<Self>,
    {
        index.get_mut(self)
    }

    /// Iterate over records in file.
    ///
    /// - First, a S0 record is returned if there is header data in the [`SRecordFile`].
    /// - Then, S3 records are returned for each data chunk, where each record is (at most)
    ///   `data_record_size` long.
    /// - Then, an S5 record is returned if the number of data records fits in 16 bites. Otherwise,
    ///   an S6 record is returned if the number of data records fits in 24 bits. Otherwise, no
    ///   count record is returned.
    /// - Finally, if a [`start_address`](`SRecordFile.start_address`) is configured in the
    ///   [`SRecordFile`] then an S7 record is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use std::str::FromStr;
    ///
    /// use srex::srecord::SRecordFile;
    ///
    /// let srecord_str = "S00F000068656C6C6F202020202000003C\n\
    ///                    S11F00007C0802A6900100049421FFF07C6C1B787C8C23783C6000003863000026\n\
    ///                    S11F001C4BFFFFE5398000007D83637880010014382100107C0803A64E800020E9\n\
    ///                    S111003848656C6C6F20776F726C642E0A0042\n\
    ///                    S5030003F9\n\
    ///                    S9030000FC";
    /// let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();
    ///
    /// for record in srecord_file.iter_records(16) {
    ///     println!("{}", record.serialize());
    /// }
    /// ```
    // TODO: Allow different file types
    pub fn iter_records(&self, data_record_size: usize) -> SRecordFileIterator {
        SRecordFileIterator {
            srecord_file: self,
            stage: SRecordFileIteratorStage::Header,
            data_chunk_index: 0,
            data_chunk_iterator: None,
            data_record_size,
            num_data_records: 0,
        }
    }

    // TODO: Documentation
    // TODO: Unit tests
    fn get_data_chunk_index(&self, address: u64, inclusive_end: bool) -> Result<usize, usize> {
        let mut left_index = 0;
        let mut right_index = self.data_chunks.len();
        loop {
            let index_diff = right_index - left_index;
            if index_diff == 0 {
                return Err(left_index);
            }
            let data_chunk = &self.data_chunks[left_index];
            let data_chunk_start_address = data_chunk.address;
            let mut data_chunk_end_address =
                data_chunk_start_address + data_chunk.data.len() as u64;
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
                let middle_index = left_index + index_diff / 2;
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

    // TODO: Tests
    pub(crate) fn get_data_chunk(&self, address: u64) -> Option<&DataChunk> {
        match self.get_data_chunk_index(address, false) {
            Ok(data_chunk_index) => Some(&self.data_chunks[data_chunk_index]),
            Err(_) => None,
        }
    }

    // TODO: Tests
    pub(crate) fn get_data_chunk_mut(&mut self, address: u64) -> Option<&mut DataChunk> {
        match self.get_data_chunk_index(address, false) {
            Ok(data_chunk_index) => Some(&mut self.data_chunks[data_chunk_index]),
            Err(_) => None,
        }
    }

    // TODO: Tests
    fn merge_data_chunks(&mut self) -> Result<(), SRecordParseError> {
        let mut index = 0;
        while index < self.data_chunks.len() - 1 {
            let current_end_address =
                self.data_chunks[index].address + self.data_chunks[index].data.len() as u64;
            let next_index = index + 1;
            let next_start_address = self.data_chunks[next_index].address;
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

impl SliceIndex<SRecordFile> for u64 {
    type Output = u8;

    fn get(self, srecord_file: &SRecordFile) -> Option<&Self::Output> {
        match srecord_file.get_data_chunk(self) {
            Some(data_chunk) => data_chunk.get(self),
            None => None,
        }
    }

    fn get_mut(self, srecord_file: &mut SRecordFile) -> Option<&mut Self::Output> {
        match srecord_file.get_data_chunk_mut(self) {
            Some(data_chunk) => data_chunk.get_mut(self),
            None => None,
        }
    }
}

impl SliceIndex<SRecordFile> for Range<u64> {
    type Output = [u8];

    fn get(self, srecord_file: &SRecordFile) -> Option<&Self::Output> {
        match srecord_file.get_data_chunk(self.start) {
            Some(data_chunk) => data_chunk.get(self),
            None => None,
        }
    }

    fn get_mut(self, srecord_file: &mut SRecordFile) -> Option<&mut Self::Output> {
        match srecord_file.get_data_chunk_mut(self.start) {
            Some(data_chunk) => data_chunk.get_mut(self),
            None => None,
        }
    }
}

impl Index<u64> for SRecordFile {
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
    fn index(&self, address: u64) -> &Self::Output {
        match address.get(self) {
            Some(data) => data,
            None => panic!("Address {address:#08X} does not exist in SRecordFile"),
        }
    }
}

impl Index<Range<u64>> for SRecordFile {
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
    fn index(&self, address_range: Range<u64>) -> &Self::Output {
        let start_address = address_range.start;
        let end_address = address_range.end;
        match address_range.get(self) {
            Some(data) => data,
            None => panic!("Address range {start_address:#08X}:{end_address:#08X} does not exist in SRecordFile"),
        }
    }
}

impl IndexMut<u64> for SRecordFile {
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
    fn index_mut(&mut self, address: u64) -> &mut Self::Output {
        match address.get_mut(self) {
            Some(data) => data,
            None => panic!("Address {address:#08X} does not exist in SRecordFile"),
        }
    }
}

impl IndexMut<Range<u64>> for SRecordFile {
    /// Performs mutable indexing in [`SRecordFile`], allowing writing using syntax
    /// `srecord_file[0x1234..0x1236] = [0x1A, 0x1B]`.
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
    /// assert_eq!(srecord_file[0x1234..0x1236], [0x01, 0x02]);
    /// srecord_file[0x1234..0x1236].fill(0xFF);
    /// assert_eq!(srecord_file[0x1234..0x1236], [0xFF, 0xFF]);
    /// ```
    ///
    /// # Panics
    ///
    /// [`index_mut`](SRecordFile::index_mut) will [`panic!`] if the input address does not exist in
    /// the [`SRecordFile`].
    fn index_mut(&mut self, address_range: Range<u64>) -> &mut Self::Output {
        let start_address = address_range.start;
        let end_address = address_range.end;
        match address_range.get_mut(self) {
            Some(data) => data,
            None => panic!("Address range {start_address:#08X}:{end_address:#08X} does not exist in SRecordFile"),
        }
    }
}

impl FromStr for SRecordFile {
    type Err = SRecordParseError;

    /// Parses an SRecord file and generates an [`SRecordFile`] containing the data in the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use std::str::FromStr;
    ///
    /// use srex::srecord::SRecordFile;
    ///
    /// let srecord_str = "S00F000068656C6C6F202020202000003C\n\
    ///                    S11F00007C0802A6900100049421FFF07C6C1B787C8C23783C6000003863000026\n\
    ///                    S11F001C4BFFFFE5398000007D83637880010014382100107C0803A64E800020E9\n\
    ///                    S111003848656C6C6F20776F726C642E0A0042\n\
    ///                    S5030003F9\n\
    ///                    S9030000FC";
    /// let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();
    /// ```
    fn from_str(srecord_str: &str) -> Result<Self, Self::Err> {
        let mut srecord_file = SRecordFile::new();

        let mut num_data_records: usize = 0;
        let mut data_buffer = [0u8; 256];

        for line in srecord_str.lines() {
            let record = Record::from_str(line, &mut data_buffer)?;
            match record {
                Record::S0Record(header_record) => match srecord_file.header_data {
                    Some(_) => {
                        return Err(SRecordParseError {
                            error_type: ErrorType::MultipleHeaderRecords,
                        })
                    }
                    None => srecord_file.header_data = Some(Vec::<u8>::from(header_record.data)),
                },
                Record::S1Record(data_record)
                | Record::S2Record(data_record)
                | Record::S3Record(data_record) => {
                    // TODO: Validate record type (no mixes?)
                    match srecord_file.get_data_chunk_index(data_record.address, true) {
                        Ok(data_chunk_index) => {
                            // Error if writing to the same address twice
                            let data_chunk = &mut srecord_file.data_chunks[data_chunk_index];
                            if data_chunk.address as usize + data_chunk.data.len()
                                != data_record.address as usize
                            {
                                return Err(SRecordParseError {
                                    error_type: ErrorType::OverlappingData,
                                });
                            }
                            data_chunk.data.extend_from_slice(data_record.data);
                        }
                        Err(data_chunk_index) => {
                            // TODO: Move out to allocation function?
                            srecord_file.data_chunks.insert(
                                data_chunk_index,
                                DataChunk {
                                    address: data_record.address,
                                    data: Vec::<u8>::from(data_record.data),
                                },
                            );
                        }
                    }
                    num_data_records += 1;
                }
                Record::S5Record(count_record) | Record::S6Record(count_record) => {
                    // TODO: Validate record count
                    // * Only last in file
                    // * Only once
                    // * Ensure it matches number of encountered data records
                    let file_num_records = count_record.record_count;
                    if num_data_records != file_num_records {
                        return Err(SRecordParseError {
                            error_type: ErrorType::CalculatedNumRecordsNotMatchingParsedNumRecords,
                        });
                    }
                }
                Record::S7Record(start_address_record)
                | Record::S8Record(start_address_record)
                | Record::S9Record(start_address_record) => {
                    if srecord_file.start_address.is_some() {
                        return Err(SRecordParseError {
                            error_type: ErrorType::MultipleStartAddresses,
                        });
                    }
                    srecord_file.start_address = Some(start_address_record.start_address);
                }
            }
        }

        // Merge data chunks
        srecord_file.merge_data_chunks()?;

        Ok(srecord_file)
    }
}

enum SRecordFileIteratorStage {
    Header,
    Data,
    Count,
    StartAddress,
    Finished,
}

pub struct SRecordFileIterator<'a> {
    srecord_file: &'a SRecordFile,
    stage: SRecordFileIteratorStage,
    data_chunk_index: usize,
    data_chunk_iterator: Option<DataChunkIterator<'a>>,
    data_record_size: usize,
    num_data_records: usize,
}

impl<'a> Iterator for SRecordFileIterator<'a> {
    type Item = Record<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stage {
            SRecordFileIteratorStage::Header => {
                self.stage = SRecordFileIteratorStage::Data;
                match self.srecord_file.header_data.as_ref() {
                    Some(header_data) => Some(Record::S0Record(HeaderRecord {
                        data: header_data.as_slice(),
                    })),
                    None => self.next(),
                }
            }
            SRecordFileIteratorStage::Data => match self.data_chunk_iterator.as_mut() {
                Some(iterator) => match iterator.next() {
                    Some(record) => {
                        self.num_data_records += 1;
                        Some(Record::S3Record(record))
                    }
                    None => {
                        self.data_chunk_index += 1;
                        self.data_chunk_iterator = None;
                        self.next()
                    }
                },
                None => {
                    match self.srecord_file.data_chunks.get(self.data_chunk_index) {
                        Some(data_chunk) => {
                            self.data_chunk_iterator =
                                Some(data_chunk.iter_records(self.data_record_size));
                        }
                        None => {
                            self.stage = SRecordFileIteratorStage::Count;
                        }
                    }
                    self.next()
                }
            },
            SRecordFileIteratorStage::Count => {
                self.stage = SRecordFileIteratorStage::StartAddress;
                if self.num_data_records < 1 << 16 {
                    Some(Record::S5Record(CountRecord {
                        record_count: self.num_data_records,
                    }))
                } else if self.num_data_records < 1 << 24 {
                    Some(Record::S6Record(CountRecord {
                        record_count: self.num_data_records,
                    }))
                } else {
                    self.next()
                }
            }
            SRecordFileIteratorStage::StartAddress => match self.srecord_file.start_address {
                Some(start_address) => {
                    self.stage = SRecordFileIteratorStage::Finished;
                    Some(Record::S7Record(StartAddressRecord { start_address }))
                }
                None => {
                    self.stage = SRecordFileIteratorStage::Finished;
                    self.next()
                }
            },
            SRecordFileIteratorStage::Finished => None,
        }
    }
}
