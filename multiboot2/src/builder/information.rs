//! Exports item [`Multiboot2InformationBuilder`].
use crate::{builder::traits::StructAsBytes, CommandLineTag};

use alloc::boxed::Box;

/// Builder to construct a valid Multiboot2 information dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Debug)]
pub struct Multiboot2InformationBuilder {}

impl Multiboot2InformationBuilder {
    pub const fn new() -> Self {
        Self {}
    }
}
