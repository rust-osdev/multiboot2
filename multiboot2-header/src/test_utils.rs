//! Test utilities.

use crate::{HeaderTagFlag, HeaderTagType};
use std::mem::size_of;

/// Returns the pointer to a variable value in memory.
#[macro_export]
macro_rules! get_ptr {
    (&$tag_ref: ident, $type: ty) => {
        (&$tag_ref) as *const $type as u64
    };
    ($tag: ident, $type: ty) => {
        get_ptr!(&$tag, $type)
    };
}

/// This macro helps to get the pointer to a specific field
/// and is helpful to check what Rust produces from the structs,
/// i.e. in scenarios with "packed" and alignments.
#[macro_export]
macro_rules! get_field_ptr {
    ($tag: ident, $field: ident, $type: ty) => {
        (&$tag.$field) as *const $type as u64
    };
}

#[repr(C)]
struct DummyHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    foo: u32,
    bar: u32,
}

#[test]
fn test_macro_get_ptr() {
    let tag = DummyHeaderTag {
        typ: HeaderTagType::End,
        flags: HeaderTagFlag::Required,
        size: 0,
        foo: 0,
        bar: 0,
    };

    let expected_base_ptr = (&tag) as *const DummyHeaderTag as u64;
    let actual_base_ptr1 = get_ptr!(tag, DummyHeaderTag);
    let actual_base_ptr2 = get_ptr!(&tag, DummyHeaderTag);
    assert_eq!(expected_base_ptr, actual_base_ptr1);
    assert_eq!(actual_base_ptr1, actual_base_ptr2);
}

#[test]
fn test_macro_get_field_ptr() {
    let tag = DummyHeaderTag {
        typ: HeaderTagType::End,
        flags: HeaderTagFlag::Required,
        size: 0,
        foo: 0,
        bar: 0,
    };

    let base_ptr = get_ptr!(tag, DummyHeaderTag);

    let prop0_ptr = get_field_ptr!(tag, typ, HeaderTagType);
    let prop1_ptr = get_field_ptr!(tag, flags, HeaderTagFlag);
    let prop2_ptr = get_field_ptr!(tag, size, u32);
    let prop3_ptr = get_field_ptr!(tag, foo, u32);
    let prop4_ptr = get_field_ptr!(tag, bar, u32);

    assert_eq!(
        prop0_ptr,
        base_ptr + 0 * size_of::<u16>() as u64 + 0 * size_of::<u32>() as u64
    );
    assert_eq!(
        prop1_ptr,
        base_ptr + 1 * size_of::<u16>() as u64 + 0 * size_of::<u32>() as u64
    );
    assert_eq!(
        prop2_ptr,
        base_ptr + 2 * size_of::<u16>() as u64 + 0 * size_of::<u32>() as u64
    );
    assert_eq!(
        prop3_ptr,
        base_ptr + 2 * size_of::<u16>() as u64 + 1 * size_of::<u32>() as u64
    );
    assert_eq!(
        prop4_ptr,
        base_ptr + 2 * size_of::<u16>() as u64 + 2 * size_of::<u32>() as u64
    );
}
