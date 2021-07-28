use crate::TagType;

/// If the image has relocatable header tag, this tag contains the image's
/// base physical address.
#[derive(Debug)]
#[repr(C)]
pub struct ImageLoadPhysAddr {
    typ: TagType,
    size: u32,
    load_base_addr: u32,
}
