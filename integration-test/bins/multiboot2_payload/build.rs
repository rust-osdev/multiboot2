fn main() {
    let linker_script = "multiboot2_payload/link.ld";
    println!("cargo:rerun-if-changed={linker_script}");
    println!("cargo:rustc-link-arg=-T{linker_script}");
}
