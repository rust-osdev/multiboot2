//! Exports item [`Multiboot2InformationBuilder`].
use crate::{builder::traits::StructAsBytes, CommandLineTag};

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Builder to construct a valid Multiboot2 information dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Debug)]
pub struct Multiboot2InformationBuilder {
    command_line_tag: Option<Box<CommandLineTag>>,
}

impl Multiboot2InformationBuilder {
    pub const fn new() -> Self {
        Self {
            command_line_tag: None,
        }
    }

    pub fn command_line_tag(&mut self, command_line_tag: Box<CommandLineTag>) {
        self.command_line_tag = Some(command_line_tag);
    }
}
