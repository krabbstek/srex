//! # Srex
//!
//! Library for parsing, reading and editing data in a
//! [Motorola S-Record file](https://en.wikipedia.org/wiki/SREC_(file_format)).
//!
//! The library provides a simple interface for reading and writing data at specific or ranges of
//! addresses:
//!
//! ```
//! use std::str::FromStr;
//! use srex::srecord::SRecordFile;
//!
//! let mut srecord_file = SRecordFile::from_str("\
//!     S00F000068656C6C6F202020202000003C\n\
//!     S321000000007C0802A6900100049421FFF07C6C1B787C8C23783C6000003863000024\n\
//!     S3210000001C4BFFFFE5398000007D83637880010014382100107C0803A64E800020E7\n\
//!     S3130000003848656C6C6F20776F726C642E0A0040\n\
//!     S5030003F9\n\
//!     S70500000000FA\n\
//! ").unwrap();
//!
//! // It is possible to access specific addresses:
//! println!("Data at address 0x1C: {}", srecord_file.get(0x1C).unwrap());
//! srecord_file[0x1C] = 0xAA;
//! println!("Data at address 0x1C is now: {}", srecord_file[0x1C]);
//!
//! // It is also possible to access address ranges:
//! assert_eq!(srecord_file[0x1C..0x1E], [0xAA, 0xFF]);
//! for (i, b) in srecord_file.get_mut(0x38..0x3C).unwrap().iter_mut().enumerate() {
//!     *b = i as u8;
//! }
//! assert_eq!(srecord_file[0x38..0x3C], [0x00, 0x01, 0x02, 0x03]);
//! ```

pub mod srecord;
