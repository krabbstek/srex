# Srex

Srex is a library for parsing [SRec](https://en.wikipedia.org/wiki/SREC_(file_format)) files.

## Example

``` rust
use std::{fs, str::FromStr};
use srex::srecord::SRecordFile;

let srecord_str = fs::read_to_string("path/to/file.s37").unwrap();
let srecord_file = SRecordFile::from_str(&srecord_str).unwrap();

// Get data at address 0x123
let x: u8 = srecord_file[0x123];
println!("Data at address 0x123: {x}");
```

:warning:  This tool is currently being built and may see large changes in future commits.
