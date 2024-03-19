use std::num::Wrapping;

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
