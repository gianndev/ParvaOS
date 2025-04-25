#![no_std] // Disables the Rust standard library for freestanding environments.
#![no_main] // Disables the default `main` function to define a custom entry point.

use core::panic::PanicInfo; // Imports the `PanicInfo` type for handling panics.

static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

// This function is called in case of panic.
#[panic_handler] // Marks the function as the panic handler for the program.
fn panic(_info: &PanicInfo) -> ! { // Defines the panic handler that takes a `PanicInfo` reference and never returns.
    loop {} // Halts the program in an infinite loop upon a panic.
}