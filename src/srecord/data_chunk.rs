// TODO: pub?
#[derive(Debug, PartialEq, Eq)]
pub struct DataChunk {
    /// Inclusive address of the start of the data chunk.
    pub address: u32,
    /// Raw data of data chunk.
    pub data: Vec<u8>,
}
