use strum_macros;
use strum_macros::Display;

#[derive(Display)]
pub enum Tag {
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
    HprofGcRootUnknown,
    HprofGcRootThreadObj,
    HprofGcRootJniGlobal,
    HprofGcRootJniLocal,
    HprofGcRootJavaFrame,
    HprofGcRootNativeStack,
    HprofGcRootStickyClass,
    HprofGcRootThreadBlock,
    HprofGcRootMonitorUsed,
    HprofGcClassDump,
    HprofGcInstanceDump,
    HprofGcObjArrayDump,
    HprofGcPrimArrayDump,
    HprofCpuSamples,
    HprofControlSettings,
    HprofHeapDumpSegment,
    HprofHeapDumpEnd,
}

impl Tag {
    pub fn id(&self) -> U1 {
        match self {
            Tag::HprofUtf8 { .. } => 0x01,
            Tag::HprofLoadClass { .. } => 0x02,
            Tag::HprofUnloadClass => 0x3,
            Tag::HprofFrame { .. } => 0x04,
            Tag::HprofTrace { .. } => 0x05,
            Tag::HprofAllocSites => 0x06,
            Tag::HprofHeapSummary => 0x07,
            Tag::HprofStartThread => 0x0A,
            Tag::HprofEndThread => 0x0B,
            Tag::HprofHeapDump => 0x0C,
            Tag::HprofCpuSamples => 0x0D,
            Tag::HprofControlSettings => 0x0E,
            Tag::HprofHeapDumpSegment => 0x1C,
            Tag::HprofHeapDumpEnd => 0x2C,
            Tag::HprofGcRootUnknown => 0xFF,
            Tag::HprofGcRootJniGlobal => 0x01,
            Tag::HprofGcRootJniLocal => 0x02,
            Tag::HprofGcRootJavaFrame => 0x03,
            Tag::HprofGcRootNativeStack => 0x04,
            Tag::HprofGcRootStickyClass => 0x05,
            Tag::HprofGcRootThreadBlock => 0x06,
            Tag::HprofGcRootMonitorUsed => 0x07,
            Tag::HprofGcRootThreadObj => 0x08,
            Tag::HprofGcClassDump => 0x20,
            Tag::HprofGcInstanceDump => 0x21,
            Tag::HprofGcObjArrayDump => 0x22,
            Tag::HprofGcPrimArrayDump => 0x23,
        }
    }
}

pub type U1 = u8;
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
pub struct HprofUtf8<'a, T> {
    id: T,
    utf8: &'a str
}
