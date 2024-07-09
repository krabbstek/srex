/// Contains error information about an error encountered in an [`SRecordFile`].
#[derive(Debug, PartialEq, Eq)]
pub struct SRecordParseError {
    /// Type of error encountered.
    pub error_type: ErrorType,
}

/// Defines different categories of errors that are checked for.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    /// Early, unexpected end of line when parsing record type (S*)
    EolWhileParsingRecordType,
    /// Early, unexpected end of line when parsing byte count
    EolWhileParsingByteCount,
    /// Early, unexpected end of line when parsing address
    EolWhileParsingAddress,
    /// Early, unexpected end of line when parsing data
    EolWhileParsingData,
    /// Early, unexpected end of line when parsing checksum
    EolWhileParsingChecksum,

    /// Line not terminated after checksum is parsed (supposed to be final byte of line
    LineNotTerminatedAfterChecksum,

    /// First character in record/line is not 'S'
    InvalidFirstCharacter,
    /// S4 record is reserved
    S4Reserved,
    /// Invalid character after 'S', e.g. 'SA'
    InvalidRecordType,

    /// Invalid byte count (e.g. invalid characters)
    InvalidByteCount,
    /// Byte count is too low for the minimum amount of bytes for record type
    ByteCountTooLowForRecordType,

    /// Invalid address (e.g. invalid characters)
    InvalidAddress,

    /// Invalid data (e.g. invalid characters)
    InvalidData,
    /// Overlapping data (data for same address encountered multiple times
    OverlappingData,

    /// Invalid checksum (e.g. invalid characters)
    InvalidChecksum,
    /// Calculated checksum from byte count, address and data does not match checksum parsed from
    /// file
    CalculatedChecksumNotMatchingParsedChecksum,

    /// Calculated/encountered number of records do not match what is configured in file
    CalculatedNumRecordsNotMatchingParsedNumRecords,

    /// Multiple header records (S0) found
    MultipleHeaderRecords,
    /// Multiple start addresses (S7|8|9) found
    MultipleStartAddresses,

    /// Record type does not match file type (e.g. S1 record in S28 file)
    RecordTypeNotMatchingFileType,
}
