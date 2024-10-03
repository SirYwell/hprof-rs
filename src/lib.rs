#![feature(bufreader_peek)]
#![feature(bufread_skip_until)]
#![feature(string_from_utf8_lossy_owned)]

use std::fs::File;

pub mod hprof_model;
pub mod reader;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::HprofReader;
    use std::fmt::Debug;
    use std::io::BufReader;

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
                Ok(_) => {}
                Err(data) => return Err(data),
            }
        }
        println!("{}", c);
        Ok(())
    }
}
