use crate::hprof_model;
use crate::hprof_model::{RecordBase, Tag, I4, U4, U8};
use hprof_model::U1;
use std::io::{Error, Read, Seek};

struct InternalHprofReader<T: Read + Seek> {
    buf_reader: T
}

pub struct HprofReader<T: Read + Seek> {
    pub identifier_size: U4,
    pub timestamp: U8,
    reader: InternalHprofReader<T>,
}

macro_rules! define_read_ux {
    ($name:ident, $type:ident, $size:expr) => {
        pub fn $name(&mut self) -> Result<$type, Error> {
            let mut buf = [0; size_of::<$type>()];
            self.buf_reader.read_exact(&mut buf).map(|_| { $type::from_be_bytes(buf)})
        }
    };
}

impl<T: Read + Seek> HprofReader<T> {
    pub fn new(buf_reader: T) -> Result<HprofReader<T>, Error> {
        let internal = InternalHprofReader::new(buf_reader);
        internal.read_hprof()
    }

    fn read_next(&mut self) -> Result<Option<Tag>, Error> {
        let tag = self.reader.read_u1()?;
        let micros = self.reader.read_u4()?;
        let body_size = self.reader.read_u4()?;
        let base = RecordBase {micros_since: micros, size_remaining: body_size};
        match tag {
            0x01 => self.read_utf8(base),
            0x02 => self.read_load_class(base),
            0x04 => self.read_frame(base),
            0x05 => self.read_trace(base),
            _ => panic!("unsupported tag: {tag}")
        }
    }

    fn read_utf8(&mut self, base: RecordBase) -> Result<Option<Tag>, Error> {
        let id = self.read_identifier()?;
        let rem = base.size_remaining - self.identifier_size;
        let mut utf8: Vec<U1> = vec![0; rem as usize];
        self.reader.buf_reader.read_exact(&mut utf8)?;
        let string = String::from_utf8(utf8).map_err(|e| Error::other(e))?;
        Ok(Some(Tag::HprofUtf8 { base, id, utf8: string }))
    }

    fn read_identifier(&mut self) -> Result<U8, Error> {
        let id = match self.identifier_size {
            4 => self.reader.read_u4()? as U8,
            8 => self.reader.read_u8()?,
            _ => panic!("unsupported id size")
        };
        Ok(id)
    }

    fn read_load_class(&mut self, base: RecordBase) -> Result<Option<Tag>, Error> {
        let class_serial_number = self.reader.read_u4()?;
        let class_object_id = self.read_identifier()?;
        let stack_trace_serial_number = self.reader.read_u4()?;
        let class_name_id = self.read_identifier()?;
        Ok(Some(Tag::HprofLoadClass { base, class_serial_number, class_object_id, stack_trace_serial_number, class_name_id}))
    }

    fn read_trace(&mut self, base: RecordBase) -> Result<Option<Tag>, Error> {
        let stack_trace_serial_number = self.reader.read_u4()?;
        let thread_serial_number = self.reader.read_u4()?;
        let number_of_frames = self.reader.read_u4()?;
        let mut stack_frame_ids = vec![0; number_of_frames as usize];
        for idx in 0..number_of_frames {
            stack_frame_ids[idx as usize] = self.read_identifier()?;
        }
        Ok(Some(Tag::HprofTrace { base, stack_trace_serial_number, thread_serial_number, stack_frame_ids}))
    }

    fn read_frame(&mut self, base: RecordBase) -> Result<Option<Tag>, Error> {
        let stack_frame_id: U8 = self.read_identifier()?;
        let method_name_id: U8 = self.read_identifier()?;
        let method_signature_id: U8 = self.read_identifier()?;
        let source_file_name_id: U8 = self.read_identifier()?;
        let class_serial_numer: U4 = self.reader.read_u4()?;
        let line_number: I4 = self.reader.read_u4()? as I4;
        Ok(Some(Tag::HprofFrame { base, stack_frame_id, method_name_id, method_signature_id, source_file_name_id, class_serial_numer, line_number }))
    }
}

impl<R: Read + Seek> Iterator for HprofReader<R> {
    type Item = Result<Tag, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next().transpose()
    }
}

impl<'a, T: Read + Seek> InternalHprofReader<T> {
    fn new(b: T) -> Self {
        Self { buf_reader: b }
    }

    fn read_hprof(mut self) -> Result<HprofReader<T>, Error> {
        let identifier_size = self.read_u4()?;
        let timestamp = self.read_u8()?;
        Ok(HprofReader { identifier_size, timestamp, reader: self })
    }

    define_read_ux!(read_u1, U1, 1);
    define_read_ux!(read_u4, U4, 4);
    define_read_ux!(read_u8, U8, 8);
}