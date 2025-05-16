#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // This line enables an unstable feature in Rust, specifically the custom_test_frameworks feature (useful because we are not using the standard library)
#![test_runner(parva_os::test_runner)] // This line specifies to use the function test_runner (defined after in the code) as the test runner instead of using Rust’s built-in test framework (which requires the standard library)
#![reexport_test_harness_main = "test_main"] // Redirect the test entry point to a function called 'test_main' instead of using the default entry point

extern crate alloc;

use core::panic::PanicInfo;  // We import this to get information about future panics
use bootloader::{entry_point, BootInfo};
use parva_os::{hlt_loop, print};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use parva_os::allocator;
    use parva_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    parva_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // parva_os::window_manager::gui();

    print!("Hello World{}", "!"); // Just an example of using the print macro
    hlt_loop()
}

// This function is called in case of panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use parva_os::println;

    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    parva_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}