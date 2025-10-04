fn main() {
    let linker_script = "link.ld";
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // relative to build.rs:
    let rerun_if_changed_path = linker_script;
    // relative to workspace root:
    let linker_arg_path = format!("{manifest_dir}/{linker_script}");

    println!("cargo:rerun-if-changed={rerun_if_changed_path}");
    println!("cargo:rustc-link-arg=-T{linker_arg_path}");
}
