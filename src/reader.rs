use std::collections::HashMap;
use crate::hprof_model;
use crate::hprof_model::{ClassInfo, FieldInfo, HeapDumpTag, RecordBase, RecordTag, Value, I4, U2, U4, U8};
use hprof_model::U1;
use std::io::{Error, ErrorKind, Read, Seek};
use crate::hprof_model::HeapDumpTag::HprofGcPrimArrayDump;

struct InternalHprofReader<T: Read + Seek> {
    buf_reader: T
}

pub struct HprofReader<T: Read + Seek> {
    pub identifier_size: U4,
    pub timestamp: U8,
    reader: InternalHprofReader<T>,
    name_cache: HashMap<U8, String>,
    class_cache: HashMap<U8, ClassInfo>,
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

    fn read_next(&mut self) -> Result<Option<RecordTag>, Error> {
        let tag = self.reader.read_u1();
        // TODO better way to detect eof?
        if tag.is_err() {
            return Ok(None) // eof
        }
        let base = self.read_base()?;
        match tag? {
            0x01 => self.read_utf8(base),
            0x02 => self.read_load_class(base),
            0x04 => self.read_frame(base),
            0x05 => self.read_trace(base),
            0x1C => self.read_heap_dump_segment(base),
            0x2C => self.read_heap_dump_end(base),
            v => panic!("unsupported tag: {:#x}", v)
        }
    }

    fn read_base(&mut self) -> Result<RecordBase, Error> {
        let micros = self.reader.read_u4()?;
        let body_size = self.reader.read_u4()?;
        let base = RecordBase { micros_since: micros, size_remaining: body_size };
        Ok(base)
    }

    fn read_utf8(&mut self, base: RecordBase) -> Result<Option<RecordTag>, Error> {
        let id = self.read_identifier()?;
        let rem = base.size_remaining - self.identifier_size;
        let mut utf8: Vec<U1> = vec![0; rem as usize];
        self.reader.buf_reader.read_exact(&mut utf8)?;
        // TODO why do we encounter invalid utf8???
        let string = String::from_utf8_lossy_owned(utf8.clone());
        // todo deal with lifetime of HprofUtf8/String properly
        self.name_cache.insert(id, string.clone());
        Ok(Some(RecordTag::HprofUtf8 { base, id, utf8: string }))
    }

    fn read_identifier(&mut self) -> Result<U8, Error> {
        let id = match self.identifier_size {
            4 => self.reader.read_u4()? as U8,
            8 => self.reader.read_u8()?,
            _ => panic!("unsupported id size")
        };
        Ok(id)
    }

    fn read_load_class(&mut self, base: RecordBase) -> Result<Option<RecordTag>, Error> {
        let class_serial_number = self.reader.read_u4()?;
        let class_object_id = self.read_identifier()?;
        let stack_trace_serial_number = self.reader.read_u4()?;
        let class_name_id = self.read_identifier()?;
        Ok(Some(RecordTag::HprofLoadClass { base, class_serial_number, class_object_id, stack_trace_serial_number, class_name_id}))
    }

    fn read_trace(&mut self, base: RecordBase) -> Result<Option<RecordTag>, Error> {
        let stack_trace_serial_number = self.reader.read_u4()?;
        let thread_serial_number = self.reader.read_u4()?;
        let number_of_frames = self.reader.read_u4()?;
        let mut stack_frame_ids = vec![0; number_of_frames as usize];
        for idx in 0..number_of_frames {
            stack_frame_ids[idx as usize] = self.read_identifier()?;
        }
        Ok(Some(RecordTag::HprofTrace { base, stack_trace_serial_number, thread_serial_number, stack_frame_ids}))
    }

    fn read_frame(&mut self, base: RecordBase) -> Result<Option<RecordTag>, Error> {
        let stack_frame_id: U8 = self.read_identifier()?;
        let method_name_id: U8 = self.read_identifier()?;
        let method_signature_id: U8 = self.read_identifier()?;
        let source_file_name_id: U8 = self.read_identifier()?;
        let class_serial_numer: U4 = self.reader.read_u4()?;
        let line_number: I4 = self.reader.read_u4()? as I4;
        Ok(Some(RecordTag::HprofFrame { base, stack_frame_id, method_name_id, method_signature_id, source_file_name_id, class_serial_numer, line_number }))
    }

    fn read_heap_dump_segment(&mut self, base: RecordBase) -> Result<Option<RecordTag>, Error> {
        let end = self.reader.buf_reader.stream_position()? + base.size_remaining as U8;
        let mut sub_records = vec![];
        while self.reader.buf_reader.stream_position()? < end {
            let id = self.reader.read_u1()?;
            let s = match id {
                0x01 => self.read_gc_root_jni_global()?,
                0x02 => self.read_gc_root_jni_local()?,
                0x03 => self.read_gc_root_java_frame()?,
                0x05 => self.read_gc_root_sticky_class()?,
                0x08 => self.read_gc_root_thread_obj()?,
                0x20 => self.read_gc_class_dump()?,
                0x21 => self.read_gc_instance_dump()?,
                0x22 => self.read_gc_obj_array_dump()?,
                0x23 => self.read_gc_prim_array_dump()?,
                _ => panic!("unknown sub-record tag {:#x}", id)
            };
            sub_records.push(s);
        }
        Ok(Some(RecordTag::HprofHeapDumpSegment {base, sub_records}))
    }

    fn read_heap_dump_end(&self, base: RecordBase) -> Result<Option<RecordTag>, Error> {
        assert_eq!(0, base.size_remaining);
        Ok(Some(RecordTag::HprofHeapDumpEnd))
    }

    fn read_gc_class_dump(&mut self) -> Result<HeapDumpTag, Error> {
        let class_object_id = self.read_identifier()?;
        let stack_trace_serial_number = self.reader.read_u4()?;
        let super_class_object_id = self.read_identifier()?;
        let class_loader_object_id = self.read_identifier()?;
        let signers_object_id = self.read_identifier()?;
        let protection_domain_object_id = self.read_identifier()?;
        let _ = self.read_identifier()?; // reserved
        let _ = self.read_identifier()?; // reserved
        let instance_size = self.reader.read_u4()?;
        let constant_pool_size = self.reader.read_u2()?;
        assert_eq!(constant_pool_size, 0, "constant pool dumping is not supported");
        let static_fields_count = self.reader.read_u2()?;
        let static_fields = self.read_fields(static_fields_count, true)?;
        let instance_field_count = self.reader.read_u2()?;
        let instance_fields = self.read_fields(instance_field_count, false)?;
        // todo deal with lifetime of class_dump properly
        let class_dump = ClassInfo {
            class_object_id,
            stack_trace_serial_number,
            super_class_object_id,
            class_loader_object_id,
            signers_object_id,
            protection_domain_object_id,
            instance_size,
            static_fields: static_fields.clone(),
            instance_fields: instance_fields.clone()
        };
        self.class_cache.insert(class_object_id, class_dump.clone());
        Ok(HeapDumpTag::HprofGcClassDump(class_dump))
    }

    fn read_fields(&mut self, field_count: U2, with_value: bool) -> Result<Vec<FieldInfo>, Error> {
        (0..field_count).map(|_| {
            let name_id = self.read_identifier()?;
            let type_tag = self.reader.read_u1()?;
            let value = if with_value {
                Some(self.read_value(type_tag)?)
            } else {
                None
            };
            Ok(FieldInfo { name_id, type_tag,value })
        }).collect()
    }

    fn read_value(&mut self, type_tag: U1) -> Result<Value, Error> {
        let v = match type_tag {
            0x01 => Value::Array { object_id: self.read_identifier()? },
            0x02 => Value::Object { object_id: self.read_identifier()? },
            0x04 => Value::Boolean(self.reader.read_u1()? != 0),
            0x05 => Value::Char(self.reader.read_u2()?),
            0x06 => Value::Float(f32::from_bits(self.reader.read_u4()?)),
            0x07 => Value::Double(f64::from_bits(self.reader.read_u8()?)),
            0x08 => Value::Byte(self.reader.read_u1()? as i8),
            0x09 => Value::Short(self.reader.read_u2()? as i16),
            0x0A => Value::Int(self.reader.read_u4()? as i32),
            0x0B => Value::Long(self.reader.read_u8()? as i64),
            _ => panic!("unsupported type tag {type_tag}")
        };
        Ok(v)
    }

    fn read_gc_root_thread_obj(&mut self) -> Result<HeapDumpTag, Error> {
        let thread_object_id = self.read_identifier()?;
        let thread_sequence_number = self.reader.read_u4()?;
        let stack_trace_sequence_number = self.reader.read_u4()?;
        Ok(HeapDumpTag::HprofGcRootThreadObj {thread_object_id, thread_sequence_number, stack_trace_sequence_number})
    }

    fn read_gc_root_java_frame(&mut self) -> Result<HeapDumpTag, Error> {
        let object_id = self.read_identifier()?;
        let thread_serial_number = self.reader.read_u4()?;
        let frame_number = self.reader.read_u4()?;
        Ok(HeapDumpTag::HprofGcRootJavaFrame { object_id, thread_serial_number, frame_number})
    }

    fn read_gc_root_jni_local(&mut self) -> Result<HeapDumpTag, Error> {
        let object_id = self.read_identifier()?;
        let thread_serial_number = self.reader.read_u4()?;
        let frame_number = self.reader.read_u4()?;
        Ok(HeapDumpTag::HprofGcRootJniLocal { object_id, thread_serial_number, frame_number})
    }

    fn read_gc_root_jni_global(&mut self) -> Result<HeapDumpTag, Error> {
        let object_id = self.read_identifier()?;
        let jni_global_ref_id = self.read_identifier()?;
        Ok(HeapDumpTag::HprofGcRootJniGlobal { object_id, jni_global_ref_id })
    }

    fn read_gc_root_sticky_class(&mut self) -> Result<HeapDumpTag, Error> {
        let object_id = self.read_identifier()?;
        Ok(HeapDumpTag::HprofGcRootStickyClass { object_id })
    }

    pub fn name(&self, id: U8) -> Option<&String> {
        self.name_cache.get(&id)
    }

    fn read_gc_instance_dump(&mut self) -> Result<HeapDumpTag, Error> {
        let object_id = self.read_identifier()?;
        let stack_trace_serial_number = self.reader.read_u4()?;
        let class_object_id = self.read_identifier()?;
        let until = self.reader.read_u4()? as u64 + self.reader.buf_reader.stream_position()?;

        let mut class = self.get_class_by_id(class_object_id)?.clone();
        let mut field_iter = class.instance_fields.iter();
        let mut values = Vec::new();
        while self.reader.buf_reader.stream_position()? < until {
            let mut field_opt = field_iter.next();
            while field_opt.is_none() {
                class = self.get_class_by_id(class.super_class_object_id)?.clone();
                field_iter = class.instance_fields.iter();
                field_opt = field_iter.next()
            }
            values.push(self.read_value(field_opt.unwrap().type_tag)?)
        }
        Ok(HeapDumpTag::HprofGcInstanceDump {object_id, stack_trace_serial_number, class_object_id, instance_field_values: values})
    }

    fn get_class_by_id(&self, object_id: U8) -> Result<&ClassInfo, Error> {
        self.class_cache.get(&object_id).ok_or(Error::other("missing class"))
    }

    fn read_gc_obj_array_dump(&mut self) -> Result<HeapDumpTag, Error> {
        let array_object_id = self.read_identifier()?;
        let stack_trace_serial_number = self.reader.read_u4()?;
        let element_count = self.reader.read_u4()?;
        let array_class_id = self.read_identifier()?;
        let mut elements = Vec::with_capacity(element_count as usize);
        for i in 0..(element_count as usize) {
            elements.insert(i, self.read_identifier()?);
        }
        Ok(HeapDumpTag::HprofGcObjArrayDump {array_object_id, stack_trace_serial_number, array_class_id, elements})
    }

    fn read_gc_prim_array_dump(&mut self) -> Result<HeapDumpTag, Error> {
        let array_object_id = self.read_identifier()?;
        let stack_trace_serial_number = self.reader.read_u4()?;
        let element_count = self.reader.read_u4()?;
        let type_tag = self.reader.read_u1()?;
        let mut elements = Vec::with_capacity(element_count as usize);
        for i in 0..(element_count as usize) {
            elements.insert(i, self.read_value(type_tag)?);
        }
        Ok(HprofGcPrimArrayDump {array_object_id, stack_trace_serial_number, elements})
    }
}

impl<R: Read + Seek> Iterator for HprofReader<R> {
    type Item = Result<RecordTag, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next().transpose()
    }
}

impl<T: Read + Seek> InternalHprofReader<T> {
    fn new(b: T) -> Self {
        Self { buf_reader: b }
    }

    fn read_hprof(mut self) -> Result<HprofReader<T>, Error> {
        let header = "JAVA PROFILE 1.0.2\0";
        let mut buf = [0u8; 19];
        self.buf_reader.read_exact(&mut buf)?;
        if buf.to_vec() != Vec::from(header) {
            return Err(Error::from(ErrorKind::InvalidInput))
        }
        let identifier_size = self.read_u4()?;
        let timestamp = self.read_u8()?;
        Ok(HprofReader {
            identifier_size,
            timestamp,
            reader: self,
            name_cache: HashMap::new(),
            class_cache: HashMap::new()
        })
    }

    define_read_ux!(read_u1, U1, 1);
    define_read_ux!(read_u2, U2, 2);
    define_read_ux!(read_u4, U4, 4);
    define_read_ux!(read_u8, U8, 8);
}