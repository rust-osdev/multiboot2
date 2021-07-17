use tags::mbi::{Tag, TagIter, TagType};
use core::fmt::{Formatter, Debug};

/// This tag indicates to the kernel what boot module was loaded along with
/// the kernel image, and where it can be found.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)] // only repr(C) would add unwanted padding near name_byte.
pub struct ModuleTag {
    typ: u32,
    size: u32,
    mod_start: u32,
    mod_end: u32,
    name_byte: u8,
}

impl ModuleTag {
    // The multiboot specification defines the module str
    // as valid utf-8, therefore this function produces
    // defined behavior
    /// Get the name of the module.
    pub fn name(&self) -> &str {
        use core::{mem, slice, str};
        let strlen = self.size as usize - mem::size_of::<ModuleTag>();
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(&self.name_byte as *const u8, strlen))
        }
    }

    /// Start address of the module.
    pub fn start_address(&self) -> u32 {
        self.mod_start
    }

    /// End address of the module
    pub fn end_address(&self) -> u32 {
        self.mod_end
    }
}

pub fn module_iter(iter: TagIter) -> ModuleIter {
    ModuleIter { iter: iter }
}

/// An iterator over all module tags.
#[derive(Clone)]
pub struct ModuleIter<'a> {
    iter: TagIter<'a>,
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = &'a ModuleTag;

    fn next(&mut self) -> Option<&'a ModuleTag> {
        self.iter
            .find(|x| x.typ == TagType::Module)
            .map(|tag| unsafe { &*(tag as *const Tag as *const ModuleTag) })
    }
}

impl <'a> Debug for ModuleIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        self.clone().for_each(|tag| {
            list.entry(&tag);
        });
        list.finish()
    }
}
