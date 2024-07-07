use crate::srecord::error::SRecordParseError;
use crate::srecord::utils::{
    calculate_checksum, parse_address, parse_byte_count, parse_data_and_checksum, parse_record_type,
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

    pub fn serialize(&self) -> String {
        // TODO: Validate byte count, address etc.?
        match self {
            Record::S0Record(header_record) => {
                // 2 address bytes, 1 checksum byte
                let byte_count = header_record.data.len() as u8 + 3;
                let checksum = calculate_checksum(&byte_count, &0, header_record.data.as_ref());
                format!(
                    "S0{byte_count:02X}0000{}{checksum:02X}",
                    hex::encode_upper(header_record.data)
                )
            }
            Record::S1Record(data_record) => {
                let address = data_record.address;
                // 2 address bytes, 1 checksum byte
                let byte_count = data_record.data.len() as u8 + 3;
                let checksum = calculate_checksum(
                    &byte_count,
                    &data_record.address,
                    data_record.data.as_ref(),
                );
                format!(
                    "S1{byte_count:02X}{address:04X}{}{checksum:02X}",
                    hex::encode_upper(data_record.data)
                )
            }
            Record::S2Record(data_record) => {
                let address = data_record.address;
                // 3 address bytes, 1 checksum byte
                let byte_count = data_record.data.len() as u8 + 4;
                let checksum = calculate_checksum(
                    &byte_count,
                    &data_record.address,
                    data_record.data.as_ref(),
                );
                format!(
                    "S2{byte_count:02X}{address:06X}{}{checksum:02X}",
                    hex::encode_upper(data_record.data)
                )
            }
            Record::S3Record(data_record) => {
                let address = data_record.address;
                // 4 address bytes, 1 checksum byte
                let byte_count = data_record.data.len() as u8 + 5;
                let checksum = calculate_checksum(
                    &byte_count,
                    &data_record.address,
                    data_record.data.as_ref(),
                );
                format!(
                    "S3{byte_count:02X}{address:08X}{}{checksum:02X}",
                    hex::encode_upper(data_record.data)
                )
            }
            Record::S5Record(count_record) => {
                // 2 address bytes, 1 checksum byte
                let byte_count = 3;
                let record_count = count_record.record_count;
                let checksum =
                    calculate_checksum(&byte_count, &(count_record.record_count as u64), &[]);
                format!("S5{byte_count:02X}{record_count:04X}{checksum:02X}")
            }
            Record::S6Record(count_record) => {
                // 3 address bytes, 1 checksum byte
                let byte_count = 4;
                let record_count = count_record.record_count as u64;
                let checksum = calculate_checksum(&byte_count, &record_count, &[]);
                format!("S6{byte_count:02X}{record_count:06X}{checksum:02X}")
            }
            Record::S7Record(start_address_record) => {
                // 4 address bytes, 1 checksum byte
                let byte_count = 5;
                let start_address = start_address_record.start_address;
                let checksum = calculate_checksum(&byte_count, &start_address, &[]);
                format!("S7{byte_count:02X}{start_address:08X}{checksum:02X}")
            }
            Record::S8Record(start_address_record) => {
                // 3 address bytes, 1 checksum byte
                let byte_count = 4;
                let start_address = start_address_record.start_address;
                let checksum = calculate_checksum(&byte_count, &start_address, &[]);
                format!("S8{byte_count:02X}{start_address:06X}{checksum:02X}")
            }
            Record::S9Record(start_address_record) => {
                // 2 address bytes, 1 checksum byte
                let byte_count = 3;
                let start_address = start_address_record.start_address;
                let checksum = calculate_checksum(&byte_count, &start_address, &[]);
                format!("S9{byte_count:02X}{start_address:04X}{checksum:02X}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CountRecord, DataRecord, HeaderRecord, Record, StartAddressRecord};

    #[test]
    fn test_serialize_s0() {
        assert_eq!(
            Record::S0Record(HeaderRecord { data: &[] }).serialize(),
            "S0030000FC"
        );
        assert_eq!(
            Record::S0Record(HeaderRecord {
                data: &[0x48, 0x44, 0x52]
            })
            .serialize(),
            "S00600004844521B"
        );
    }

    #[test]
    fn test_serialize_s1() {
        assert_eq!(
            Record::S1Record(DataRecord {
                address: 0,
                data: &[]
            })
            .serialize(),
            "S1030000FC",
        );
        assert_eq!(
            Record::S1Record(DataRecord {
                address: 0x1234,
                data: &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
            })
            .serialize(),
            "S10912340102030405069B",
        );
    }

    #[test]
    fn test_serialize_s2() {
        assert_eq!(
            Record::S2Record(DataRecord {
                address: 0,
                data: &[]
            })
            .serialize(),
            "S204000000FB",
        );
        assert_eq!(
            Record::S2Record(DataRecord {
                address: 0x123456,
                data: &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
            })
            .serialize(),
            "S20A12345601020304050644",
        );
    }

    #[test]
    fn test_serialize_s3() {
        assert_eq!(
            Record::S3Record(DataRecord {
                address: 0,
                data: &[]
            })
            .serialize(),
            "S30500000000FA",
        );
        assert_eq!(
            Record::S3Record(DataRecord {
                address: 0x12345678,
                data: &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
            })
            .serialize(),
            "S30B12345678010203040506CB",
        );
    }

    #[test]
    fn test_serialize_s5() {
        assert_eq!(
            Record::S5Record(CountRecord { record_count: 0 }).serialize(),
            "S5030000FC",
        );
        assert_eq!(
            Record::S5Record(CountRecord {
                record_count: 0x1234,
            })
            .serialize(),
            "S5031234B6",
        );
    }

    #[test]
    fn test_serialize_s6() {
        assert_eq!(
            Record::S6Record(CountRecord { record_count: 0 }).serialize(),
            "S604000000FB",
        );
        assert_eq!(
            Record::S6Record(CountRecord {
                record_count: 0x123456,
            })
            .serialize(),
            "S6041234565F",
        );
    }

    #[test]
    fn test_serialize_s7() {
        assert_eq!(
            Record::S7Record(StartAddressRecord { start_address: 0 }).serialize(),
            "S70500000000FA",
        );
        assert_eq!(
            Record::S7Record(StartAddressRecord {
                start_address: 0x12345678,
            })
            .serialize(),
            "S70512345678E6",
        );
    }

    #[test]
    fn test_serialize_s8() {
        assert_eq!(
            Record::S8Record(StartAddressRecord { start_address: 0 }).serialize(),
            "S804000000FB",
        );
        assert_eq!(
            Record::S8Record(StartAddressRecord {
                start_address: 0x123456,
            })
            .serialize(),
            "S8041234565F",
        );
    }

    #[test]
    fn test_serialize_s9() {
        assert_eq!(
            Record::S9Record(StartAddressRecord { start_address: 0 }).serialize(),
            "S9030000FC",
        );
        assert_eq!(
            Record::S9Record(StartAddressRecord {
                start_address: 0x1234,
            })
            .serialize(),
            "S9031234B6",
        );
    }
}
