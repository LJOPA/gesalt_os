#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use core::arch::asm;
use bootloader_api::info::FrameBufferInfo;
extern crate alloc;

#[cfg(test)]
use bootloader_api::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

pub mod interrupts;
pub mod gdt;
pub mod serial;
pub mod framebuffer;
pub mod logger;
// pub mod vga_buffer;
pub mod memory;
pub mod allocator;
pub mod task;
pub mod scheduler;
pub mod userspace;
pub mod dummy_driver;

pub fn init() {
    gdt::init();
    unsafe { gdt::init_syscalls() };
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Initialize a text-based logger using the given pixel-based framebuffer as output.  
pub fn init_logger(
    info: FrameBufferInfo,
    framebuffer: &'static mut [u8]
) {
    let logger = logger::LOGGER.get_or_init(move || {
        logger::LockedLogger::new(
            framebuffer,
            info,
        )
    });
    log::set_logger(logger).expect("logger already set");
    log::set_max_level(log::LevelFilter::Info);
    log::info!("Framebuffer info: {:?}", info);
}

#[naked]
pub unsafe extern "C" fn syscall_wrapper() {
    asm!(
        "nop",
        "iretq",
        options(noreturn)
    )
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}