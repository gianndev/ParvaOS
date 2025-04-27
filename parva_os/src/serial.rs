use spin::Mutex;
use lazy_static::lazy_static;
use uart_16550::SerialPort; // Imports the 'SerialPort' type, which provides an interface for working with serial communication over the UART 16550 protocol

// Initialize something only once (in this case initialize SerialPort)
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// To print stuff directly on the terminal instead of using the QEMU screen
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

// Prints to the host through the serial interface.
#[macro_export] // Indicates that the following function will be available in all the files of the project and not only in this one
// serial_print is like the print functions in 'vga_buffer.rs', but it prints to the terminal instead of the QEMU screen
macro_rules! serial_print {
    ($($arg:tt)*) => { 
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
// This is similar to serial_print, but adds a newline character at the end of the output (this is exactly what happens with print and println)
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}