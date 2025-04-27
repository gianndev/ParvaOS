#![no_std] // Don't link the Rust standard library
#![no_main] // Disables the fact that the first function to be started must be called 'main'
#![feature(custom_test_frameworks)] // This line enables an unstable feature in Rust, specifically the custom_test_frameworks feature (useful because we are not using the standard library)
#![test_runner(parva_os::test_runner)] // This line specifies to use the function test_runner (defined after in the code) as the test runner instead of using Rust’s built-in test framework (which requires the standard library)
#![reexport_test_harness_main = "test_main"] // Redirect the test entry point to a function called 'test_main' instead of using the default entry point

use parva_os::println;
use core::panic::PanicInfo;  // We import this to get information about future panics

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    parva_os::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
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