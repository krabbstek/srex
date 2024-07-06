use std::ops::Range;

use crate::srecord::slice_index::SliceIndex;

/// A contiguous chunk of data at a specific address.
///
/// [`DataChunk`]s are intended to be the largest contiguous ranges of data, allowing flexible
/// slicing of contiguous data.
#[derive(Debug, PartialEq, Eq)]
pub struct DataChunk {
    /// Start address of the [`DataChunk`]. The first byte of the data is located at this address.
    pub address: u64,
    /// Raw contiguous data of data chunk, starting at `address`.
    pub data: Vec<u8>,
}

impl DataChunk {
    /// Returns inclusive start address of [`DataChunk`]. Same as `address`.
    pub fn start_address(&self) -> u64 {
        self.address
    }

    /// Exclusive end address of [`DataChunk`]. This is the first address in ascending order after
    /// [`DataChunk`] that does not contain any data inside the chunk.
    pub fn end_address(&self) -> u64 {
        self.address + self.data.len() as u64
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
    /// use srex::srecord::DataChunk;
    ///
    /// let data_chunk = DataChunk{
    ///     address: 0x10000,
    ///     data: vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05],
    /// };
    /// assert!(data_chunk.get(0x10000).is_some());
    /// assert!(data_chunk.get(0x10006).is_none());
    /// assert_eq!(data_chunk.get(0x10001..0x10003).unwrap(), &[0x01u8, 0x02u8]);
    /// assert!(data_chunk.get(0x10000..0x10006).is_some());
    /// assert!(data_chunk.get(0x10000..0x10007).is_none());
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
    /// use srex::srecord::DataChunk;
    ///
    /// let mut data_chunk = DataChunk{
    ///     address: 0x10000,
    ///     data: vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05],
    /// };
    /// assert!(data_chunk.get_mut(0x10000).is_some());
    /// assert_eq!(*data_chunk.get_mut(0x10000).unwrap(), 0x00u8);
    /// *data_chunk.get_mut(0x10000).unwrap() = 0x10;
    /// assert_eq!(*data_chunk.get_mut(0x10000).unwrap(), 0x10u8);
    /// ```
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<Self>,
    {
        index.get_mut(self)
    }
}

impl SliceIndex<DataChunk> for u64 {
    type Output = u8;

    fn get(self, data_chunk: &DataChunk) -> Option<&u8> {
        match self.checked_sub(data_chunk.address) {
            Some(index) => data_chunk.data.get(index as usize),
            None => None,
        }
    }

    fn get_mut(self, data_chunk: &mut DataChunk) -> Option<&mut u8> {
        match self.checked_sub(data_chunk.address) {
            Some(index) => data_chunk.data.get_mut(index as usize),
            None => None,
        }
    }
}

impl SliceIndex<DataChunk> for Range<u64> {
    type Output = [u8];

    fn get(self, data_chunk: &DataChunk) -> Option<&[u8]> {
        match self.start.checked_sub(data_chunk.address) {
            Some(start_index) => match self.end.checked_sub(data_chunk.address) {
                Some(end_index) => data_chunk
                    .data
                    .get(start_index as usize..end_index as usize),
                None => None,
            },
            None => None,
        }
    }

    fn get_mut(self, data_chunk: &mut DataChunk) -> Option<&mut [u8]> {
        match self.start.checked_sub(data_chunk.address) {
            Some(start_index) => match self.end.checked_sub(data_chunk.address) {
                Some(end_index) => data_chunk
                    .data
                    .get_mut(start_index as usize..end_index as usize),
                None => None,
            },
            None => None,
        }
    }
}
