#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;
extern crate alloc;

pub mod serial;
pub mod vga;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod window_manager;
pub mod parva_fs;
pub mod process;
pub mod time;
pub mod ata;
pub mod keyboard;

use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    gdt::init();
    interrupts::init();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    keyboard::init();
    memory::init(boot_info);
    ata::init();
    parva_fs::ParvaFS::init();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(_exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0x604);
        port.write(0x2000u16); // QEMU shutdown command
    }
    hlt_loop(); // Halt the CPU after sending the shutdown signal
}

// Reboots the system by sending a reset command to the keyboard controller.
pub fn reboot() -> ! {
    use x86_64::instructions::port::Port;

    // The standard method on x86 is to write 0xFE to port 0x64
    unsafe {
        let mut port: Port<u8> = Port::new(0x64);
        port.write(0xFE);
    }

    // In case the above doesn't trigger a reboot, halt the CPU.
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::entry_point;

#[cfg(test)]
use core::panic::PanicInfo;

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let csi_color = kernel::console::Style::color("LightRed");
    let csi_reset = kernel::console::Style::reset();
    print!("{}failed{}\n\n", csi_color, csi_reset);
    print!("{}\n\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}