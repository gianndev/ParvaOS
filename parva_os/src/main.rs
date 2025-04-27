#![no_std] // Don't link the Rust standard library
#![no_main] // Disables the fact that the first function to be started must be called 'main'
#![feature(custom_test_frameworks)] // This line enables an unstable feature in Rust, specifically the custom_test_frameworks feature (useful because we are not using the standard library)
#![test_runner(parva_os::test_runner)] // This line specifies to use the function test_runner (defined after in the code) as the test runner instead of using Rust’s built-in test framework (which requires the standard library)
#![reexport_test_harness_main = "test_main"] // Redirect the test entry point to a function called 'test_main' instead of using the default entry point

use parva_os::println;
use core::panic::PanicInfo;  // We import this to get information about future panics
use bootloader::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use parva_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::{structures::paging::Page, VirtAddr};
    println!("Hello World{}", "!");

    parva_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    parva_os::hlt_loop();
}

// This function is called in case of panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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