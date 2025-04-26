#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo; // Imports the `PanicInfo` type for handling panics.

mod vga;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main(); // Calls the test main function if the test configuration is enabled.

    loop {}
}

// The panic_handler attribute defines the function that the compiler should invoke when a panic occurs. 
// The standard library provides its own panic handler function, but in a no_std environment we need to define one ourselves:
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running tests...");
    for test in tests {
        test();
    }
    println!("All tests passed!");
}

#[test_case]
fn trivial_assertion() {
    print!("trivial_assertion...");
    assert_eq!(1, 1);
    println!("[ok]");
}