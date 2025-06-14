use alloc::boxed::Box;
use alloc::vec::Vec;
use elf_rs::{ElfFile, ProgramHeaderEntry, ProgramType};
use log::{debug, info};
use multiboot2::{
    BootLoaderNameTag, CommandLineTag, EFIMemoryAreaType, MaybeDynSized, MemoryArea,
    MemoryAreaType, MemoryMapTag, ModuleTag, SmbiosTag,
};

fn get_free_mmap_areas(
    mbi: &multiboot2::BootInformation,
) -> Vec<(u64 /* start */, u64 /* size */)> {
    match (mbi.memory_map_tag(), mbi.efi_memory_map_tag()) {
        (Some(mmt), None) => mmt
            .memory_areas()
            .iter()
            .filter(|ma| ma.typ() == MemoryAreaType::Available)
            .map(|ma| (ma.start_address(), ma.size()))
            .collect::<alloc::vec::Vec<_>>(),
        (_, Some(mmt)) => mmt
            .memory_areas()
            .filter(|ma| ma.ty == EFIMemoryAreaType::CONVENTIONAL)
            .map(|ma| (ma.phys_start, ma.page_count * 4096))
            .collect::<alloc::vec::Vec<_>>(),
        _ => panic!("No usable memory map"),
    }
}

fn assert_load_segment_fits_into_memory(
    start: u64,
    size: u64,
    free_areas: &[(u64 /* start */, u64 /* size */)],
) {
    let end = start + size;
    let range = free_areas
        .iter()
        .find(|(a_start, a_size)| start >= *a_start && end <= a_start + a_size);
    if let Some(range) = range {
        debug!("Can load load segment (0x{start:x?}, {size:x?}) into free memory area {range:#x?}");
    } else {
        panic!("Can't load load segment  (0x{start:x?}, {size:x?}) into any area!");
    }
}

/// Loads the first module into memory. Assumes that the module is a ELF file.
/// The handoff is performed according to the Multiboot2 spec.
pub fn load_module(mbi: &multiboot2::BootInformation) -> ! {
    let mut modules = mbi.module_tags();

    // Load the ELF from the Multiboot1 boot module.
    let elf_mod = modules.next().expect("Should have payload");
    let elf_bytes = unsafe {
        core::slice::from_raw_parts(
            elf_mod.start_address() as *const u64 as *const u8,
            elf_mod.module_size() as usize,
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

    // Load the load segments into memory (at their corresponding link address).
    {
        let free_areas = get_free_mmap_areas(mbi);
        elf.program_header_iter()
            .inspect(|ph| assert_load_segment_fits_into_memory(ph.vaddr(), ph.memsz(), &free_areas))
            .filter(|ph| ph.ph_type() == ProgramType::LOAD)
            .for_each(|ph| {
                map_memory(ph);
            });
    }

    // Currently, the MBI is not enriched with "real" information as requested.
    // Subject here is not to write a feature-complete bootloader but to test
    // that the basic data structures are usable.

    // build MBI
    let mbi = multiboot2::Builder::new()
        .bootloader(BootLoaderNameTag::new("mb2_integrationtest_chainloader"))
        .cmdline(CommandLineTag::new("chainloaded YEAH"))
        // random non-sense memory map
        .mmap(MemoryMapTag::new(&[MemoryArea::new(
            0,
            0xffffffff,
            MemoryAreaType::Reserved,
        )]))
        .add_module(ModuleTag::new(
            elf_mod.start_address(),
            elf_mod.end_address(),
            elf_mod.cmdline().unwrap(),
        ))
        // Test that we can add SmbiosTag multiple times.
        .add_smbios(SmbiosTag::new(1, 1, &[1, 2, 3]))
        .add_smbios(SmbiosTag::new(2, 3, &[4, 5, 6]))
        .build();

    let mbi = Box::leak(mbi);

    log::info!(
        "Handing over to ELF: {}",
        elf_mod.cmdline().unwrap_or("<unknown>")
    );

    // handoff
    unsafe {
        core::arch::asm!(
        "jmp *%ecx",
        in("eax") multiboot2::MAGIC,
        in("ebx") mbi.as_ptr(),
        in("ecx") elf.entry_point() as u32,
        options(noreturn, att_syntax));
    }
}

/// Blindly copies the LOAD segment content at its desired address in physical
/// address space. The loader assumes that the addresses to not clash with the
/// loader (or anything else).
fn map_memory(ph: ProgramHeaderEntry) {
    debug!("Mapping LOAD segment {ph:#?}");
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
