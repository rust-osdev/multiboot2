//! Module for tag types.
//!
//! The relevant exports of this module are [`TagTypeRaw`] and [`TagType`].

multiboot2_common::raw_type! {
    /// ABI compatible representation of the type of a boot information tag.
    ///
    /// This type matches the binary representation (`u32`) and corresponds
    /// to the `typ`/`type` field of a Multiboot2 [`TagHeader`]. It can
    /// easily be created from or converted to [`TagType`].
    ///
    /// [`TagHeader`]: crate::TagHeader
    pub struct TagTypeRaw(u32);

    /// The type of a boot information tag.
    ///
    /// This assigns each possible value a specific semantic according to the
    /// Multiboot2 spec. Custom tag types `> 21` are mapped to
    /// [`TagType::Custom`]; the spec doesn't explicitly allow or disallow
    /// them.
    ///
    /// This is a higher level abstraction for [`TagTypeRaw`] and **not
    /// binary compatible** with it.
    pub enum TagType {
        /// Tag `0`: Marks the end of the tags.
        End = 0,
        /// Tag `1`: Additional command line string.
        /// For example `''` or `'--my-custom-option foo --provided by_grub`, if
        /// your GRUB config contains `multiboot2 /boot/multiboot2-binary.elf --my-custom-option foo --provided by_grub`
        Cmdline = 1,
        /// Tag `2`: Name of the bootloader, e.g. 'GRUB 2.04-1ubuntu44.2'
        BootLoaderName = 2,
        /// Tag `3`: Additional Multiboot modules, which are BLOBs provided in
        /// memory. For example an initial ram disk with essential drivers.
        Module = 3,
        /// Tag `4`: `mem_lower` and `mem_upper` indicate the amount of lower and
        /// upper memory, respectively, in kilobytes. Lower memory starts at
        /// address 0, and upper memory starts at address 1 megabyte. The maximum
        /// possible value for lower memory is 640 kilobytes. The value returned
        /// for upper memory is maximally the address of the first upper memory
        /// hole minus 1 megabyte. It is not guaranteed to be this value.
        ///
        /// This tag may not be provided by some bootloaders on EFI platforms if
        /// EFI boot services are enabled and available for the loaded image (EFI
        /// boot services not terminated tag exists in Multiboot2 information
        /// structure).
        BasicMeminfo = 4,
        /// Tag `5`: This tag indicates which BIOS disk device the bootloader
        /// loaded the OS image from. If the OS image was not loaded from a BIOS
        /// disk, then this tag must not be present. The operating system may use
        /// this field as a hint for determining its own root device, but is not
        /// required to.
        Bootdev = 5,
        /// Tag `6`: Memory map. The map provided is guaranteed to list all
        /// standard RAM that should be available for normal use. This type however
        /// includes the regions occupied by kernel, mbi, segments and modules.
        /// Kernel must take care not to overwrite these regions.
        ///
        /// This tag may not be provided by some bootloaders on EFI platforms if
        /// EFI boot services are enabled and available for the loaded image (EFI
        /// boot services not terminated tag exists in Multiboot2 information
        /// structure).
        Mmap = 6,
        /// Tag `7`: Contains the VBE control information returned by the VBE
        /// Function `0x00` and VBE mode information returned by the VBE Function
        /// `0x01`, respectively. Note that VBE 3.0 defines another protected mode
        /// interface which is incompatible with the old one. If you want to use
        /// the new protected mode interface, you will have to find the table
        /// yourself.
        Vbe = 7,
        /// Tag `8`: Framebuffer.
        Framebuffer = 8,
        /// Tag `9`: This tag contains section header table from an ELF kernel, the
        /// size of each entry, number of entries, and the string table used as the
        /// index of names. They correspond to the `shdr_*` entries (`shdr_num`,
        /// etc.) in the Executable and Linkable Format (ELF) specification in the
        /// program header.
        ElfSections = 9,
        /// Tag `10`: APM table. See Advanced Power Management (APM) BIOS Interface
        /// Specification, for more information.
        Apm = 10,
        /// Tag `11`: This tag contains pointer to i386 EFI system table.
        Efi32 = 11,
        /// Tag `12`: This tag contains pointer to amd64 EFI system table.
        Efi64 = 12,
        /// Tag `13`: This tag contains a copy of SMBIOS tables as well as their
        /// version.
        Smbios = 13,
        /// Tag `14`: Also called "AcpiOld" in other multiboot2 implementations.
        AcpiV1 = 14,
        /// Tag `15`: Refers to version 2 and later of Acpi.
        /// Also called "AcpiNew" in other multiboot2 implementations.
        AcpiV2 = 15,
        /// Tag `16`: This tag contains network information in the format specified
        /// as DHCP. It may be either a real DHCP reply or just the configuration
        /// info in the same format. This tag appears once
        /// per card.
        Network = 16,
        /// Tag `17`: This tag contains EFI memory map as per EFI specification.
        /// This tag may not be provided by some bootloaders on EFI platforms if
        /// EFI boot services are enabled and available for the loaded image (EFI
        /// boot services not terminated tag exists in Multiboot2 information
        /// structure).
        EfiMmap = 17,
        /// Tag `18`: This tag indicates ExitBootServices wasn't called.
        EfiBs = 18,
        /// Tag `19`: This tag contains pointer to EFI i386 image handle. Usually
        /// it is bootloader image handle.
        Efi32Ih = 19,
        /// Tag `20`: This tag contains pointer to EFI amd64 image handle. Usually
        /// it is bootloader image handle.
        Efi64Ih = 20,
        /// Tag `21`: This tag contains image load base physical address. The spec
        /// tells *"It is provided only if image has relocatable header tag."* but
        /// experience showed that this is not true for at least GRUB 2.
        LoadBaseAddr = 21,
    }
}
