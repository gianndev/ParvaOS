#![no_std] // Disables the Rust standard library for freestanding environments.
#![no_main] // Disables the default `main` function to define a custom entry point.

use core::panic::PanicInfo; // Imports the `PanicInfo` type for handling panics.

mod vga;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    loop {}
}

// The panic_handler attribute defines the function that the compiler should invoke when a panic occurs. 
// The standard library provides its own panic handler function, but in a no_std environment we need to define one ourselves:
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}