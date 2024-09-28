#![feature(bufreader_peek)]
#![feature(bufread_skip_until)]
#![feature(string_from_utf8_lossy_owned)]

use std::fs::File;

pub mod hprof_model;
mod reader;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;
    use std::fmt::Debug;
    use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
    use crate::hprof_model::{Tag, U8};
    use crate::reader::{HprofReader};

    #[test]
    fn it_works() -> std::io::Result<()> {
        let file = File::open("heap.hprof")?;
        let mut reader = BufReader::new(file);
        let header = "JAVA PROFILE 1.0.2";
        let buf = reader.peek(header.len())?;
        if buf != header.as_bytes() {
            return Err(Error::from(ErrorKind::InvalidInput))
        }
        let zero_index = reader.skip_until(0)?;
        assert_eq!(header.len() + 1, zero_index);
        let mut hprof_reader = HprofReader::new(reader)?;
        let identifier_size = hprof_reader.identifier_size;
        assert_eq!(identifier_size, 8);
        let timestamp = hprof_reader.timestamp;
        println!("{:?}", timestamp);
        let mut name_lookup: HashMap<U8, String> = HashMap::new();
        while let Some(rec) = hprof_reader.next() {
            match rec {
                Ok(Tag::HprofUtf8 { id, utf8, .. }) => {
                    name_lookup.insert(id, utf8);
                }
                Ok(Tag::HprofLoadClass { class_name_id, .. }) => {
                    let name = name_lookup.get(&class_name_id);
                    if name.is_none() {
                        println!("no name???")
                    }
                }
                Ok(other) => { println!("{}", other)}
                Err(_) => {}
            }
        }
        Ok(())
    }
}
