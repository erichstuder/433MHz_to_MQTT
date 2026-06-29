use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    create_linker_script();
}

fn create_linker_script() {
    // start of flash memory
    // see: https://pip-assets.raspberrypi.com/categories/814-rp2040/documents/RP-008371-DS-1-rp2040-datasheet.pdf #2.2.1
    const XIP_BASE: usize = 0x1000_0000;

    // BOOT2 length
    // Copied from example: https://github.com/embassy-rs/embassy/blob/main/examples/rp/memory.x
    const BOOT2_LENGTH: usize = 0x100;

    // 2 MB of on-board flash memory
    // see: https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html#pico1
    const FLASH_MEMORY_LENGTH: usize = 2 * 1024 * 1024;
    const FLASH_LENGTH: usize = FLASH_MEMORY_LENGTH - BOOT2_LENGTH - DEVICE_DATA_LENGTH;

    const DEVICE_DATA_ORIGIN: usize = XIP_BASE + FLASH_MEMORY_LENGTH - DEVICE_DATA_LENGTH;
    const DEVICE_DATA_LENGTH: usize = 0x2000;

    let memory_x = format!(
"MEMORY {{
    BOOT2       : ORIGIN = {boot2_origin:#x}, LENGTH = {boot2_length:#x}
    FLASH       : ORIGIN = {flash_origin:#x}, LENGTH = {flash_length:#x}
    DEVICE_DATA : ORIGIN = {device_data_origin:#x}, LENGTH = {device_data_length:#x}
    RAM         : ORIGIN = 0x20000000, LENGTH = 264K
}}",
        boot2_origin       = XIP_BASE,
        boot2_length       = BOOT2_LENGTH,
        flash_origin       = XIP_BASE + BOOT2_LENGTH,
        flash_length       = FLASH_LENGTH,
        device_data_origin = DEVICE_DATA_ORIGIN,
        device_data_length = DEVICE_DATA_LENGTH,
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out_dir.join("memory.x"))
        .unwrap()
        .write_all(memory_x.as_bytes())
        .unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rustc-env=FLASH_MEMORY_LENGTH={}", FLASH_MEMORY_LENGTH);
    // Relative origin means the beginning of the Flash Memory is address 0.
    println!("cargo:rustc-env=DEVICE_DATA_RELATIVE_ORIGIN={}", DEVICE_DATA_ORIGIN - XIP_BASE);
    println!("cargo:rustc-env=DEVICE_DATA_LENGTH={}", DEVICE_DATA_LENGTH);
}
