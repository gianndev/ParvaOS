#![no_std] // Disables the Rust standard library for freestanding environments.
#![no_main] // Disables the default `main` function to define a custom entry point.

use core::panic::PanicInfo; // Imports the `PanicInfo` type for handling panics.

#[unsafe(no_mangle)] // Prevents the compiler from mangling the name of the `_start` function.
pub extern "C" fn _start() -> ! { // Defines the entry point `_start` with C calling convention and no return.
    loop {} // Enters an infinite loop since there's no OS to terminate the program.
}

// This function is called in case of panic.
#[cfg(not(test))] // Ensures this panic handler is excluded when running tests.
#[panic_handler] // Marks the function as the panic handler for the program.
fn panic(_info: &PanicInfo) -> ! { // Defines the panic handler that takes a `PanicInfo` reference and never returns.
    loop {} // Halts the program in an infinite loop upon a panic.
}