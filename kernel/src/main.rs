#![no_std]
#![no_main]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader_api::{BootInfo, entry_point};
use bootloader_api::config::{BootloaderConfig, Mapping};
use kernel::{serial_println, init_logger};
use kernel::task::{Task, keyboard, executor::Executor};
use log::{info, warn};
use acpi::{self, AcpiTables};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    info!("async number: {}", number);
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use kernel::allocator;
    use kernel::scheduler;
    use kernel::memory::{self, BootInfoFrameAllocator, usable_mem_count, PHYS_MEMORY_OFFSET};
    use x86_64::VirtAddr;
    // use x86_64::structures::paging::PhysFrame;
    // use x86_64::structures::idt::InterruptStackFrame;
    // use kernel::dummy_driver::dummy_driver;

    unsafe { PHYS_MEMORY_OFFSET = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap())};
    
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    init_logger(
        framebuffer.info(),
        framebuffer.buffer_mut()
    );
    
    info!("Kernel starting... ");

    kernel::init();

    let mut mapper = unsafe { memory::init(PHYS_MEMORY_OFFSET) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    
    info!("Physical memory mapping offset {:#x}", boot_info.physical_memory_offset.into_option().unwrap());
    info!("Kernel entry point at virtual address {:p}", kernel_main as *const () );
    // unsafe {  info!("Kernel entry point at physical address {:#x}", translate_addr(VirtAddr::from_ptr(kernel_main as *const ()), PHYS_MEMORY_OFFSET).unwrap().as_u64()); }
    
    // print_mem_map(&boot_info.memory_regions);
    info!("{} bytes usable", usable_mem_count(&boot_info.memory_regions));
    
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // Checking for ACPI table by checking for an address provided by the bootloader
    let rsdp_addr = boot_info.rsdp_addr.into_option();
    match rsdp_addr {
        Some(_) => info!("ACPI Table found: {:#x}", rsdp_addr.unwrap()),
        _ => warn!("No ACPI Table found!")
    }
    
    // AcpiTables::from_rsdp(handler, rsdp_address)

    scheduler::init_sched();

    #[cfg(test)]
    test_main();
    
    x86_64::instructions::interrupts::enable();
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();    
    serial_println!("{}", info);
    kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}