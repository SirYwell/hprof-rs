use std::io::{BufReader, Error, Read};
use tags::U1;
use crate::tags;
use crate::tags::{U4, U8};

pub struct HprofReader<T:> {
    buf_reader: BufReader<T>
}

macro_rules! define_read_ux {
    ($name:ident, $type:ident, $size:expr) => {
        pub fn $name(&mut self) -> Result<$type, Error> {
            let mut buf = [0; size_of::<$type>()];
            self.buf_reader.read_exact(&mut buf).map(|_| { $type::from_be_bytes(buf)})
        }
    };
}

impl<T: Read> HprofReader<T> {
    pub fn new(b: BufReader<T>) -> HprofReader<T> {
        HprofReader { buf_reader: b }
    }

    define_read_ux!(read_u1, U1, 1);
    define_read_ux!(read_u4, U4, 4);
    define_read_ux!(read_u8, U8, 8);
}