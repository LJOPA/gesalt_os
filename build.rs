use std::path::Path;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir_osstring = std::env::var_os("OUT_DIR").unwrap();
    let out_dir: &Path = out_dir_osstring.as_ref();
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel_osstring = std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap();
    let kernel: &Path = kernel_osstring.as_ref();

    // create an UEFI disk image (optional)
    let uefi_path_buff = out_dir.join("uefi.img");
    let uefi_path = uefi_path_buff.as_path();
    bootloader::UefiBoot::new(&kernel).create_disk_image(uefi_path).unwrap();

    // create a BIOS disk image (optional)
    let bios_path_buff = out_dir.join("bios.img");
    let bios_path = bios_path_buff.as_path();
    bootloader::BiosBoot::new(&kernel).create_disk_image(bios_path).unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}