
#[derive(Debug)]
#[repr(packed)] // repr(C) would add unwanted padding before first_section
pub struct BootLoaderNameTag {
    typ: u32,
    size: u32,
    string: u8,
}

impl BootLoaderNameTag {
    pub fn name(&self) -> &str {
        unsafe { ::core::str::from_utf8_unchecked(::core::slice::from_raw_parts((&self.string) as *const u8, self.size as usize - 8)) }
    }
}
