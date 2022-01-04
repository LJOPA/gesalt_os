#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![feature(const_mut_refs)]
#![test_runner(nu_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use nu_os::{println, print};
use core::panic::PanicInfo;
use bootloader::{boot_info::BootInfo, entry_point};
use nu_os::task::Task;
use nu_os::task::executor::Executor;
use nu_os::task::keyboard;

entry_point!(kernel_main);

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use nu_os::allocator;
    use nu_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    print!("Kernel starting... ");
    nu_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_regions)
    };
    
    println!("Done");
    println!("Kernel entry point at virtual address {:p}", kernel_main as *const () );
    
    // unsafe { println!("Kernel entry point at physical address {:#x}", translate_addr(VirtAddr::from_ptr(kernel_main as *const ()), phys_mem_offset).unwrap().as_u64()); }
    
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    
        
    x86_64::instructions::interrupts::enable();    
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
    
    #[cfg(test)]
    test_main();
    
    // nu_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();    
    println!("{}", info);
    nu_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    nu_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}