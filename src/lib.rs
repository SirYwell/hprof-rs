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
    use super::*;
    use crate::hprof_model::RecordTag;
    use crate::reader::HprofReader;
    use std::fmt::Debug;
    use std::io::{BufRead, BufReader, Error, ErrorKind};

    #[test]
    fn it_works() -> std::io::Result<()> {
        let file = File::open("heap.hprof")?;
        let reader = BufReader::new(file);
        let mut hprof_reader = HprofReader::new(reader)?;
        let identifier_size = hprof_reader.identifier_size;
        assert_eq!(identifier_size, 8);
        let timestamp = hprof_reader.timestamp;
        println!("{:?}", timestamp);
        let mut c = 0u64;
        while let Some(res) = hprof_reader.next() {
            c += 1;
            match res {
                Ok(rec) => {
                    match rec {
                        RecordTag::HprofUtf8 { id, utf8, .. } => {
                        }
                        RecordTag::HprofLoadClass { class_name_id, .. } => {
                            let name = hprof_reader.name(class_name_id);
                            if name.is_none() {
                                println!("no name???")
                            }
                        }
                        RecordTag::HprofFrame { .. } => {}
                        RecordTag::HprofTrace { .. } => {}
                        other => {
                            println!("{}", other)
                        }
                    }
                }
                Err(data) => {
                    println!("error {data}");
                    return Ok(())
                }
            }
        }
        println!("{}", c);
        Ok(())
    }
}
