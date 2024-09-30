use strum_macros;
use strum_macros::Display;

// top-level tags
#[derive(Display)]
pub enum RecordTag {
    HprofUtf8 {
        base: RecordBase,
        id: U8,
        utf8: String,
    },
    HprofLoadClass {
        base: RecordBase,
        class_serial_number: U4,
        class_object_id: U8,
        stack_trace_serial_number: U4,
        class_name_id: U8,
    },
    HprofUnloadClass,
    HprofFrame {
        base: RecordBase,
        stack_frame_id: U8,
        method_name_id: U8,
        method_signature_id: U8,
        source_file_name_id: U8,
        class_serial_numer: U4,
        line_number: I4,
    },
    HprofTrace {
        base: RecordBase,
        stack_trace_serial_number: U4,
        thread_serial_number: U4,
        stack_frame_ids: Vec<U8>,
    },
    HprofAllocSites,
    HprofStartThread,
    HprofEndThread,
    HprofHeapSummary,
    HprofHeapDump,
    HprofCpuSamples,
    HprofControlSettings,
    HprofHeapDumpSegment {
        base: RecordBase,
        sub_records: Vec<HeapDumpTag>
    },
    HprofHeapDumpEnd,
}

// sub-record tags in HPROF_HEAP_DUMP and HPROF_HEAP_DUMP_SEGMENT
pub enum HeapDumpTag {
    HprofGcRootUnknown,
    HprofGcRootThreadObj {
        thread_object_id: U8,
        thread_sequence_number: U4,
        stack_trace_sequence_number: U4
    },
    HprofGcRootJniGlobal {
        object_id: U8,
        jni_global_ref_id: U8
    },
    HprofGcRootJniLocal {
        object_id: U8,
        thread_serial_number: U4,
        frame_number: U4 // "frame # in stack trace (-1 for empty)" ...???
    },
    HprofGcRootJavaFrame {
        object_id: U8,
        thread_serial_number: U4,
        frame_number: U4 // "frame # in stack trace (-1 for empty)" ...???
    },
    HprofGcRootNativeStack,
    HprofGcRootStickyClass {
        object_id: U8
    },
    HprofGcRootThreadBlock,
    HprofGcRootMonitorUsed,
    HprofGcClassDump(ClassInfo),
    HprofGcInstanceDump {
        object_id: U8,
        stack_trace_serial_number: U4,
        class_object_id: U8,
        instance_field_values: Vec<Value>
    },
    HprofGcObjArrayDump {
        array_object_id: U8,
        stack_trace_serial_number: U4,
        array_class_id: U8,
        elements: Vec<U8>
    },
    HprofGcPrimArrayDump {
        array_object_id: U8,
        stack_trace_serial_number: U4,
        elements: Vec<Value> // TODO only one type is valid here, can we represent that?
    },
}

impl RecordTag {
    pub fn id(&self) -> U1 {
        match self {
            RecordTag::HprofUtf8 { .. } => 0x01,
            RecordTag::HprofLoadClass { .. } => 0x02,
            RecordTag::HprofUnloadClass => 0x3,
            RecordTag::HprofFrame { .. } => 0x04,
            RecordTag::HprofTrace { .. } => 0x05,
            RecordTag::HprofAllocSites => 0x06,
            RecordTag::HprofHeapSummary => 0x07,
            RecordTag::HprofStartThread => 0x0A,
            RecordTag::HprofEndThread => 0x0B,
            RecordTag::HprofHeapDump => 0x0C,
            RecordTag::HprofCpuSamples => 0x0D,
            RecordTag::HprofControlSettings => 0x0E,
            RecordTag::HprofHeapDumpSegment { .. } => 0x1C,
            RecordTag::HprofHeapDumpEnd => 0x2C,
        }
    }
}

impl HeapDumpTag {
    pub fn id(&self) -> U1 {
        match self {
            HeapDumpTag::HprofGcRootUnknown => 0xFF,
            HeapDumpTag::HprofGcRootJniGlobal { .. } => 0x01,
            HeapDumpTag::HprofGcRootJniLocal { .. } => 0x02,
            HeapDumpTag::HprofGcRootJavaFrame { .. } => 0x03,
            HeapDumpTag::HprofGcRootNativeStack => 0x04,
            HeapDumpTag::HprofGcRootStickyClass { .. } => 0x05,
            HeapDumpTag::HprofGcRootThreadBlock => 0x06,
            HeapDumpTag::HprofGcRootMonitorUsed => 0x07,
            HeapDumpTag::HprofGcRootThreadObj { .. } => 0x08,
            HeapDumpTag::HprofGcClassDump(_) => 0x20,
            HeapDumpTag::HprofGcInstanceDump { .. } => 0x21,
            HeapDumpTag::HprofGcObjArrayDump { .. } => 0x22,
            HeapDumpTag::HprofGcPrimArrayDump { .. } => 0x23,
        }
    }
}

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;
pub type I4 = i32;
pub type U8 = u64;
pub trait Identifier {
    fn size() -> usize;
}
impl Identifier for U4 {
    fn size() -> usize {
        (Self::BITS >> 3) as usize
    }
}

impl Identifier for U8 {
    fn size() -> usize {
        (Self::BITS >> 3) as usize
    }
}
pub struct RecordBase {
    pub(crate) micros_since: U4,
    pub(crate) size_remaining: U4
}

#[derive(Clone)]
pub struct ClassInfo {
    pub class_object_id: U8,
    pub stack_trace_serial_number: U4,
    pub super_class_object_id: U8,
    pub class_loader_object_id: U8,
    pub signers_object_id: U8,
    pub protection_domain_object_id: U8,
    pub instance_size: U4,
    pub static_fields: Vec<FieldInfo>,
    pub instance_fields: Vec<FieldInfo>
}
#[derive(Clone)]
pub struct FieldInfo {
    pub name_id: U8,
    pub type_tag: U1,
    pub value: Option<Value>
}

#[derive(Clone)]
pub enum Value {
    Object {
        object_id: U8
    },
    Array {
        object_id: U8
    },
    Byte(i8),
    Char(u16),
    Short(i16),
    Float(f32),
    Double(f64),
    Int(i32),
    Long(i64),
    Boolean(bool)
}
