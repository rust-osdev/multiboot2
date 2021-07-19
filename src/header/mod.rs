#![allow(missing_docs)]

mod address;
mod console;
mod end;
mod entry_efi_32;
mod entry_efi_64;
mod entry_header;
mod framebuffer;
mod information_request;
mod module_alignment;
mod relocatable;
mod tags;
mod uefi_bs;
mod mb2_header;


pub use self::address::*;
pub use self::console::*;
pub use self::end::*;
pub use self::entry_efi_32::*;
pub use self::entry_efi_64::*;
pub use self::entry_header::*;
pub use self::framebuffer::*;
pub use self::information_request::*;
pub use self::module_alignment::*;
pub use self::relocatable::*;
pub use self::tags::*;
pub use self::uefi_bs::*;
pub use self::mb2_header::*;


/// Value must be present in multiboot2 header.
pub const MULTIBOOT2_HEADER_MAGIC: u32 = 0xe85250d6;

/// Loads the data on the given address as Multiboot2 header.
/// The address must be 8-byte aligned (see specification).
pub unsafe fn load_mb2_header<'a>(addr: usize) -> Multiboot2Header<'a> {
    assert_ne!(0, addr, "null pointer");
    assert_eq!(addr % 8, 0, "must be 8-byte aligned, see multiboot spec");
    let ptr = addr as *const Multiboot2HeaderInner;
    assert!((*ptr).verify_checksum(), "checksum invalid!");
    let reference = &*ptr;
    Multiboot2Header::new(reference)
}
