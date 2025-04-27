#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // This line enables an unstable feature in Rust, specifically the custom_test_frameworks feature (useful because we are not using the standard library)
#![test_runner(parva_os::test_runner)] // This line specifies to use the function test_runner (defined after in the code) as the test runner instead of using Rust’s built-in test framework (which requires the standard library)
#![reexport_test_harness_main = "test_main"] // Redirect the test entry point to a function called 'test_main' instead of using the default entry point

extern crate alloc;

use parva_os::println;
use core::panic::PanicInfo;  // We import this to get information about future panics
use bootloader::{entry_point, BootInfo};
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use parva_os::allocator;
    use parva_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    parva_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

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