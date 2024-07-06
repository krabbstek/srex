use crate::srecord::error::SRecordParseError;
use crate::srecord::utils::{
    parse_address, parse_byte_count, parse_data_and_checksum, parse_record_type,
};
use crate::srecord::RecordType;

pub struct HeaderRecord<'a> {
    pub data: &'a [u8],
}

pub struct DataRecord<'a> {
    pub address: u64,
    pub data: &'a [u8],
}

pub struct CountRecord {
    pub record_count: usize,
}

pub struct StartAddressRecord {
    pub start_address: u64,
}

pub enum Record<'a> {
    S0Record(HeaderRecord<'a>),
    S1Record(DataRecord<'a>),
    S2Record(DataRecord<'a>),
    S3Record(DataRecord<'a>),
    S5Record(CountRecord),
    S6Record(CountRecord),
    S7Record(StartAddressRecord),
    S8Record(StartAddressRecord),
    S9Record(StartAddressRecord),
}

impl Record<'_> {
    #[inline]
    pub fn from_str<'a>(s: &str, data: &'a mut [u8]) -> Result<Record<'a>, SRecordParseError> {
        let record_type = parse_record_type(s)?;
        let byte_count = parse_byte_count(s)?;
        let address = parse_address(s, &record_type)?;
        let num_data_types = record_type.num_data_bytes(byte_count as usize);
        parse_data_and_checksum(s, &record_type, &byte_count, &address, data)?;
        let data = &data[..num_data_types];

        match record_type {
            RecordType::S0 => Ok(Record::S0Record(HeaderRecord { data })),
            RecordType::S1 => {
                // TODO: Validate that data does not extend past max 16-bit address
                Ok(Record::S1Record(DataRecord { address, data }))
            }
            RecordType::S2 => {
                // TODO: Validate that data does not extend past max 24-bit address
                Ok(Record::S2Record(DataRecord { address, data }))
            }
            RecordType::S3 => {
                // TODO: Validate that data does not extend past max 32-bit address
                Ok(Record::S3Record(DataRecord { address, data }))
            }
            RecordType::S5 => Ok(Record::S5Record(CountRecord {
                record_count: address as usize,
            })),
            RecordType::S6 => Ok(Record::S6Record(CountRecord {
                record_count: address as usize,
            })),
            RecordType::S7 => Ok(Record::S7Record(StartAddressRecord {
                start_address: address,
            })),
            RecordType::S8 => Ok(Record::S7Record(StartAddressRecord {
                start_address: address,
            })),
            RecordType::S9 => Ok(Record::S7Record(StartAddressRecord {
                start_address: address,
            })),
        }
    }
}
