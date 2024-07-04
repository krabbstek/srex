use std::ops::Range;

use crate::srecord::get::Get;

#[derive(Debug, PartialEq, Eq)]
pub struct DataChunk {
    /// Inclusive address of the start of the data chunk.
    pub address: u64,
    /// Raw data of data chunk.
    pub data: Vec<u8>,
}

impl Get<u64> for DataChunk {
    type Output = u8;

    fn get(&self, address: u64) -> Option<&Self::Output> {
        match address.checked_sub(self.address) {
            Some(index) => self.data.get(index as usize),
            None => None,
        }
    }

    fn get_mut(&mut self, address: u64) -> Option<&mut Self::Output> {
        match address.checked_sub(self.address) {
            Some(index) => self.data.get_mut(index as usize),
            None => None,
        }
    }
}

impl Get<Range<u64>> for DataChunk {
    type Output = [u8];

    fn get(&self, address_range: Range<u64>) -> Option<&Self::Output> {
        match address_range.start.checked_sub(self.address) {
            Some(start_index) => match address_range.end.checked_sub(self.address) {
                Some(end_index) => self.data.get(start_index as usize..end_index as usize),
                None => None,
            },
            None => None,
        }
    }

    fn get_mut(&mut self, address_range: Range<u64>) -> Option<&mut Self::Output> {
        match address_range.start.checked_sub(self.address) {
            Some(start_index) => match address_range.end.checked_sub(self.address) {
                Some(end_index) => self.data.get_mut(start_index as usize..end_index as usize),
                None => None,
            },
            None => None,
        }
    }
}
