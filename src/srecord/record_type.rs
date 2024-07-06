use std::fmt;

use crate::srecord::Record;

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

    pub fn num_data_bytes(&self, byte_count: usize) -> usize {
        byte_count - (self.num_address_bytes() + 1)
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
}
