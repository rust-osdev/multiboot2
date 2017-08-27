use core::marker::PhantomData;

#[repr(C)]
pub struct Tag {
    pub typ: u32,
    pub size: u32,
    // tag specific fields
}

pub struct TagIter<'a> {
    pub current: *const Tag,
    pub phantom: PhantomData<&'a Tag>,
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        match unsafe{&*self.current} {
            &Tag{typ:0, size:8} => None, // end tag
            tag => {
                // go to next tag
                let mut tag_addr = self.current as usize;
                tag_addr += ((tag.size + 7) & !7) as usize; //align at 8 byte
                self.current = tag_addr as *const _;

                Some(tag)
            },
        }
    }
}
