#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // This line enables an unstable feature in Rust, specifically the custom_test_frameworks feature (useful because we are not using the standard library)
#![test_runner(parva_os::test_runner)] // This line specifies to use the function test_runner (defined after in the code) as the test runner instead of using Rust’s built-in test framework (which requires the standard library)
#![reexport_test_harness_main = "test_main"] // Redirect the test entry point to a function called 'test_main' instead of using the default entry point

extern crate alloc;

use core::panic::PanicInfo;  // We import this to get information about future panics
use bootloader::{entry_point, BootInfo};
use parva_os::{hlt_loop, print, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    parva_os::init(boot_info);
    parva_os::window_manager::gui();

    // Uncomment the following lines to have terminal mode instead of GUI mode (and comment the GUI line above)
    // println!("Hello from Parva OS!");
    // parva_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop{}
}