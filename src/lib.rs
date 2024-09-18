#![feature(bufreader_peek)]
#![feature(bufread_skip_until)]

use std::fs::File;

pub mod tags;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::io::{BufRead, BufReader, Error, ErrorKind};

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
        Ok(())
    }
}
