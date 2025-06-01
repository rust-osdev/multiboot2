use crate::verify::{print_elf_info, print_memory_map, print_module_info};
use multiboot2::BootInformation;

pub fn run(mbi: &BootInformation) -> anyhow::Result<()> {
    basic_sanity_checks(mbi)?;
    print_memory_map(mbi)?;
    print_module_info(mbi)?;
    print_elf_info(mbi)?;
    Ok(())
}

fn basic_sanity_checks(mbi: &BootInformation) -> anyhow::Result<()> {
    // Some basic sanity checks
    let bootloader_name = mbi
        .boot_loader_name_tag()
        .ok_or("No bootloader tag")
        .map_err(anyhow::Error::msg)?
        .name()
        .map_err(anyhow::Error::msg)?;
    let cmdline = mbi
        .command_line_tag()
        .ok_or("No cmdline tag")
        .map_err(anyhow::Error::msg)?
        .cmdline()
        .map_err(anyhow::Error::msg)?;
    assert!(bootloader_name.starts_with("GRUB 2.") || bootloader_name.starts_with("Limine"));
    assert_eq!(cmdline, "some kernel cmdline");

    Ok(())
}
