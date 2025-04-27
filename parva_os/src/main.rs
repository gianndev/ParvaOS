#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{arch::asm, panic::PanicInfo}; // Imports the `PanicInfo` type for handling panics.

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
    exit_qemu(QemuExitCode::QEMU_EXIT_SUCCESS);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial_assertion...");
    assert_eq!(1, 1);
    println!("[ok]");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    QEMU_EXIT_SUCCESS = 0x10,
    QEMU_EXIT_FAILURE = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        // The `out` instruction is used to send data to a port in x86 assembly.
        // The `0xf4` port is used by QEMU to exit the emulator.
        asm!("out dx, eax", in("dx") 0xf4, in("eax") exit_code as u32);
    }
}