#![feature(bufreader_peek)]
#![feature(bufread_skip_until)]

use std::fs::File;

pub mod tags;
mod reader;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
    use crate::reader::HprofReader;

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
        let mut hprof_reader = HprofReader::new(reader);
        let identifier_size = hprof_reader.read_u4()?;
        assert_eq!(identifier_size, 8);
        let timestamp = hprof_reader.read_u8()?;
        println!("{:?}", timestamp);
        Ok(())
    }
}
