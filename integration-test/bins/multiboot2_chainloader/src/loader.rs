use elf_rs::{ElfFile, ProgramHeaderEntry, ProgramType};
use multiboot2::{
    BootLoaderNameTag, CommandLineTag, MemoryArea, MemoryAreaType, MemoryMapTag, ModuleTag,
};

/// Loads the first module into memory. Assumes that the module is a ELF file.
/// The handoff is performed according to the Multiboot2 spec.
pub fn load_module(mut modules: multiboot::information::ModuleIter) -> ! {
    // Load the ELF from the Multiboot1 boot module.
    let elf_mod = modules.next().expect("Should have payload");
    let elf_bytes = unsafe {
        core::slice::from_raw_parts(
            elf_mod.start as *const u64 as *const u8,
            (elf_mod.end - elf_mod.start) as usize,
        )
    };
    let elf = elf_rs::Elf32::from_bytes(elf_bytes).expect("Should be valid ELF");

    // Check if a header is present.
    {
        let hdr = multiboot2_header::Multiboot2Header::find_header(elf_bytes)
            .unwrap()
            .expect("Should have Multiboot2 header");
        let hdr =
            unsafe { multiboot2_header::Multiboot2Header::load(hdr.0.as_ptr().cast()) }.unwrap();
        log::info!("Multiboot2 header:\n{hdr:#?}");
    }


    // Map the load segments into memory (at their corresponding link).
    {
        let elf = elf_rs::Elf32::from_bytes(elf_bytes).expect("Should be valid ELF");
        elf.program_header_iter()
            .filter(|ph| ph.ph_type() == ProgramType::LOAD)
            .for_each(|ph| {
                map_memory(ph);
            });
    }

    // Currently, the MBI is not enriched with "real" information as requested.
    // Subject here is not to write a feature-complete bootloader but to test
    // that the basic data structures are usable.

    // build MBI
    let mbi = multiboot2::builder::InformationBuilder::new()
        .bootloader_name_tag(BootLoaderNameTag::new("mb2_integrationtest_chainloader"))
        .command_line_tag(CommandLineTag::new("chainloaded YEAH"))
        // random non-sense memory map
        .memory_map_tag(MemoryMapTag::new(&[MemoryArea::new(
            0,
            0xffffffff,
            MemoryAreaType::Reserved,
        )]))
        .add_module_tag(ModuleTag::new(
            elf_mod.start as u32,
            elf_mod.end as u32,
            elf_mod.string.unwrap(),
        ))
        .build();

    log::info!(
        "Handing over to ELF: {}",
        elf_mod.string.unwrap_or("<unknown>")
    );

    // handoff
    unsafe {
        core::arch::asm!(
        "jmp *%ecx",
        in("eax") multiboot2::MAGIC,
        in("ebx") mbi.as_ptr() as u32,
        in("ecx") elf.entry_point() as u32,
        options(noreturn, att_syntax));
    }
}

/// Blindly copies the LOAD segment content at its desired address in physical
/// address space. The loader assumes that the addresses to not clash with the
/// loader (or anything else).
fn map_memory(ph: ProgramHeaderEntry) {
    log::debug!("Mapping LOAD segment {ph:#?}");
    let dest_ptr = ph.vaddr() as *mut u8;
    let content = ph.content().expect("Should have content");
    unsafe { core::ptr::copy(content.as_ptr(), dest_ptr, content.len()) };
    let dest_ptr = unsafe { dest_ptr.add(ph.filesz() as usize) };

    // Zero .bss memory
    for _ in 0..(ph.memsz() - ph.filesz()) {
        unsafe {
            core::ptr::write(dest_ptr, 0);
        }
    }
}
