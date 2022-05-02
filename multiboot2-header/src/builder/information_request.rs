use super::traits::StructAsBytes;
use crate::InformationRequestHeaderTag;
use crate::{HeaderTagFlag, MbiTagType};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::mem::size_of;

/// Helper to build the dynamically sized [`InformationRequestHeaderTag`]
/// at runtime. The information request tag has a dedicated builder because this way one
/// can dynamically attach several requests to it. Otherwise, the number of requested tags
/// must be known at compile time.
#[derive(Debug)]
#[cfg(feature = "builder")]
pub struct InformationRequestHeaderTagBuilder {
    flag: HeaderTagFlag,
    // information requests (irs)
    irs: BTreeSet<MbiTagType>,
}

#[cfg(feature = "builder")]
impl InformationRequestHeaderTagBuilder {
    /// New builder.
    pub fn new(flag: HeaderTagFlag) -> Self {
        Self {
            irs: BTreeSet::new(),
            flag,
        }
    }

    /// Returns the expected length of the information request tag,
    /// when the `build`-method gets called.
    pub fn expected_len(&self) -> usize {
        let basic_header_size = size_of::<InformationRequestHeaderTag<0>>();
        let req_tags_size = self.irs.len() * size_of::<MbiTagType>();
        basic_header_size + req_tags_size
    }

    /// Adds an [`MbiTagType`] to the information request.
    pub fn add_ir(mut self, tag: MbiTagType) -> Self {
        self.irs.insert(tag);
        self
    }

    /// Adds multiple [`MbiTagType`] to the information request.
    pub fn add_irs(mut self, tags: &[MbiTagType]) -> Self {
        self.irs.extend(tags);
        self
    }

    /// Builds the bytes of the dynamically sized information request header.
    pub fn build(self) -> Vec<u8> {
        let expected_len = self.expected_len();
        let mut data = Vec::with_capacity(expected_len);

        let basic_tag = InformationRequestHeaderTag::<0>::new(
            self.flag,
            [],
            // we put the expected length here already, because in the next step we write
            // all the tags into the byte array. We can't know this during compile time,
            // therefore N is 0.
            Some(expected_len as u32),
        );
        data.extend(basic_tag.struct_as_bytes());
        #[cfg(debug_assertions)]
        {
            let basic_tag_size = size_of::<InformationRequestHeaderTag<0>>();
            assert_eq!(
                data.len(),
                basic_tag_size,
                "the vector must be as long as the basic tag!"
            );
        }

        for tag in &self.irs {
            let bytes: [u8; 4] = (*tag as u32).to_ne_bytes();
            data.extend(&bytes);
        }

        debug_assert_eq!(
            data.len(),
            expected_len,
            "the byte vector must be as long as the expected size of the struct"
        );

        data
    }
}
#[cfg(test)]
mod tests {
    use crate::builder::information_request::InformationRequestHeaderTagBuilder;
    use crate::{HeaderTagFlag, InformationRequestHeaderTag, MbiTagType};

    #[test]
    fn test_builder() {
        let builder = InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
            .add_ir(MbiTagType::EfiMmap)
            .add_ir(MbiTagType::BootLoaderName)
            .add_ir(MbiTagType::Cmdline);
        // type(u16) + flags(u16) + size(u32) + 3 tags (u32)
        assert_eq!(builder.expected_len(), 2 + 2 + 4 + 3 * 4);
        let tag = builder.build();
        let tag = unsafe {
            (tag.as_ptr() as *const InformationRequestHeaderTag<3>)
                .as_ref()
                .unwrap()
        };
        assert_eq!(tag.flags(), HeaderTagFlag::Required);
        // type(u16) + flags(u16) + size(u32) + 3 tags (u32)
        assert_eq!(tag.size(), 2 + 2 + 4 + 3 * 4);
        assert_eq!(tag.dynamic_requests_size(), 3);
        assert!(tag.requests().contains(&MbiTagType::EfiMmap));
        assert!(tag.requests().contains(&MbiTagType::BootLoaderName));
        assert!(tag.requests().contains(&MbiTagType::Cmdline));
        assert_eq!(tag.requests().len(), 3);
        assert!(!tag.requests().contains(&MbiTagType::AcpiV1));
        println!("{:#?}", tag);
    }
}
