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
    parva_os::init(boot_info);
    parva_os::window_manager::gui();
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