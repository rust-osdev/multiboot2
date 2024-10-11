mod bootloader;
mod chainloader;

use alloc::format;
use alloc::vec::Vec;
use multiboot2::BootInformation;

pub fn run(mbi: &BootInformation) -> anyhow::Result<()> {
    println!("MBI: {mbi:#x?}");
    println!();

    let bootloader = mbi
        .boot_loader_name_tag()
        .ok_or("No bootloader tag")
        .map_err(anyhow::Error::msg)?
        .name()
        .map_err(anyhow::Error::msg)?;

    if bootloader.to_lowercase().contains("grub") {
        log::info!("loaded by GRUB");
        bootloader::run(mbi)?;
    } else if bootloader.to_lowercase().contains("limine") {
        log::info!("loaded by Limine");
        bootloader::run(mbi)?;
    } else {
        log::info!("loaded by chainloader");
        chainloader::run(mbi)?;
    }

    Ok(())
}

pub(self) fn print_memory_map(mbi: &BootInformation) -> anyhow::Result<()> {
    let memmap = mbi
        .memory_map_tag()
        .ok_or("Should have memory map")
        .map_err(anyhow::Error::msg)?;
    println!("Memory Map:");
    memmap.memory_areas().iter().for_each(|e| {
        println!(
            "  0x{:010x} - 0x{:010x} ({:.3} MiB {:?})",
            e.start_address(),
            e.end_address(),
            e.size() as f32 / 1024.0 / 1024.0,
            e.typ()
        );
    });
    println!();
    Ok(())
}

pub(self) fn print_elf_info(mbi: &BootInformation) -> anyhow::Result<()> {
    let sections_iter = mbi
        .elf_sections()
        .ok_or("Should have elf sections")
        .map_err(anyhow::Error::msg)?;
    println!("ELF sections:");
    for s in sections_iter {
        let typ = format!("{:?}", s.section_type());
        let flags = format!("{:?}", s.flags());
        let name = s.name().map_err(anyhow::Error::msg)?;
        println!(
            "  {:<13} {:<17} {:<22} 0x{:010x} 0x{:010x} {:>5.2} MiB align={}",
            name,
            typ,
            flags,
            s.start_address(),
            s.end_address(),
            s.size() as f32 / 1024.0,
            s.addralign(),
        );
    }
    println!();
    Ok(())
}

pub(self) fn print_module_info(mbi: &BootInformation) -> anyhow::Result<()> {
    let modules = mbi.module_tags().collect::<Vec<_>>();
    if modules.len() != 1 {
        Err(anyhow::Error::msg("Should have exactly one boot module"))?
    }
    let module = modules.first().unwrap();
    let module_cmdline = module.cmdline().map_err(anyhow::Error::msg)?;

    let allowed_module_cmdlines = ["Limine bootloader config", "multiboot2_payload"];
    assert!(
        allowed_module_cmdlines
            .iter()
            .any(|&str| module_cmdline == str),
        "The module cmdline must be one of {allowed_module_cmdlines:?} but is {module_cmdline}"
    );

    println!("Modules:");
    println!(
        "  0x{:010x} - 0x{:010x} ({} B, cmdline='{}')",
        module.start_address(),
        module.end_address(),
        module.module_size(),
        module_cmdline
    );
    println!(" bootloader cfg passed as boot module:");
    let grup_cfg_ptr = module.start_address() as *const u32 as *const u8;
    let grub_cfg =
        unsafe { core::slice::from_raw_parts(grup_cfg_ptr, module.module_size() as usize) };

    // In the Limine bootflow case, we pass the config as module with it. This
    // is not done for the chainloaded case.
    if let Ok(str) = core::str::from_utf8(grub_cfg) {
        println!("=== file begin ===");
        for line in str.lines() {
            println!("    > {line}");
        }
        println!("=== file end ===");
        println!();
    }

    Ok(())
}
