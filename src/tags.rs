
pub enum Tag<'a, T> {
    HprofUtf8(HprofUtf8<'a, T>),
    HprofLoadClass,
    HprofUnloadClass,
    HprofFrame,
    HprofTrace,
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
pub type U1 = u8;
pub type U4 = u32;
pub type U8 = u64;
union ID {
    id: u32,
}
pub struct RecordBase {
    tag: U1,
    micros: U4,
    size: U4
}
pub struct HprofUtf8<'a, T> {
    id: T,
    utf8: &'a str
}
