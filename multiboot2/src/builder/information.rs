//! Exports item [`Multiboot2InformationBuilder`].
use crate::builder::traits::StructAsBytes;
use crate::{
    BasicMemoryInfoTag, BootLoaderNameTag, CommandLineTag, ElfSectionsTag, FramebufferTag,
    MemoryMapTag, ModuleTag,
};

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Builder to construct a valid Multiboot2 information dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Debug)]
pub struct Multiboot2InformationBuilder {
    basic_memory_info_tag: Option<BasicMemoryInfoTag>,
    boot_loader_name_tag: Option<Box<BootLoaderNameTag>>,
    command_line_tag: Option<Box<CommandLineTag>>,
    elf_sections_tag: Option<Box<ElfSectionsTag>>,
    framebuffer_tag: Option<Box<FramebufferTag>>,
    memory_map_tag: Option<Box<MemoryMapTag>>,
    module_tags: Vec<Box<ModuleTag>>,
}

impl Multiboot2InformationBuilder {
    pub const fn new() -> Self {
        Self {
            basic_memory_info_tag: None,
            boot_loader_name_tag: None,
            command_line_tag: None,
            elf_sections_tag: None,
            framebuffer_tag: None,
            memory_map_tag: None,
            module_tags: Vec::new(),
        }
    }

    pub fn basic_memory_info_tag(&mut self, basic_memory_info_tag: BasicMemoryInfoTag) {
        self.basic_memory_info_tag = Some(basic_memory_info_tag)
    }

    pub fn bootloader_name_tag(&mut self, boot_loader_name_tag: Box<BootLoaderNameTag>) {
        self.boot_loader_name_tag = Some(boot_loader_name_tag);
    }

    pub fn command_line_tag(&mut self, command_line_tag: Box<CommandLineTag>) {
        self.command_line_tag = Some(command_line_tag);
    }

    pub fn elf_sections_tag(&mut self, elf_sections_tag: Box<ElfSectionsTag>) {
        self.elf_sections_tag = Some(elf_sections_tag);
    }

    pub fn framebuffer_tag(&mut self, framebuffer_tag: Box<FramebufferTag>) {
        self.framebuffer_tag = Some(framebuffer_tag);
    }

    pub fn memory_map_tag(&mut self, memory_map_tag: Box<MemoryMapTag>) {
        self.memory_map_tag = Some(memory_map_tag);
    }

    pub fn add_module_tag(&mut self, module_tag: Box<ModuleTag>) {
        self.module_tags.push(module_tag);
    }
}
