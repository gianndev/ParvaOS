// VGA Text Mode Explanation:
//
// The VGA (Video Graphics Array) text mode is a display mode used by computers to render text on the screen
// Introduced by IBM in 1987, VGA became a standard for computer graphics and text display
// In VGA text mode, the screen is divided into a grid of characters, typically 80 columns wide and 25 rows high
// Each position in this grid can hold a single character, which is represented by a byte that specifies the ASCII code of the character
// Additionally, another byte is used to define the character's attributes, such as the foreground and background colors
// The VGA text mode supports a palette of 16 colors for both text and background, allowing for various combinations and enhancing the readability of the text
// This mode was widely used in early IBM-compatible computers and DOS-based systems because it required fewer resources compared to more advanced graphical modes
//
// In VGA text mode, each character on the screen is represented by 16 bits (2 bytes):
// - The first byte (8 bits) represents the ASCII code of the character. This determines which character is displayed at that position
// - The second byte (8 bits) represents character's attributes: 0-3 bits for foreground color (with 4 bits you can select 16 different numbers, each one corresponding to a different color), 4-6 for background color (with 3 bits you can select 8 different numbers, each one corresponding to a different color) and the 7th bit to decide if the character will blink on the screen
//
//      COLOR   |   FORE    |   BACK   
//  ------------|-----------|----------
//      Black   |   0000    |   000
//      Blue    |   0001    |   001
//      Green   |   0010    |   010
//      Cyan    |   0011    |   011
//      Red     |   0100    |   100
//      Magenta |   0101    |   101
//      Brown   |   0110    |   110
//   Light Gray |   0111    |   111
//    Dark Gray |   1000    |   N/A
//   Light Blue |   1001    |   N/A
//  Light Green |   1010    |   N/A
//   Light Cyan |   1011    |   N/A
//    Light Red |   1100    |   N/A
//      Pink    |   1101    |   N/A
//      Yellow  |   1110    |   N/A
//      White   |   1111    |   N/A

use core::fmt; // The fmt module provides essential stuff for text output, like 'fmt::Write', 'fmt::Display' and 'fmt::Debug'
use lazy_static::lazy_static; // lazy_static is used to initialize commands to be done only once at the beginning of the program and not in future (like a "bootloader" of the code)
use spin::Mutex; // Mutex ensures only one thread or execution context can access a particular resource at a time. In this case, it ensures that only one part of the code can access the WRITER at once, which is crucial for preventing concurrent access issues
use volatile::Volatile; // Useful to make sure that all commands are executed in the right order, following the code
use alloc::string::String;
use alloc::format;

// Let's define a code to be executed only once
lazy_static! {
    // 'pub' means public, and it makes WRITER accessible outside the current Rust file
    // 'static' defines WRITER as a static variable, meaning it is allocated once for the entire runtime of the OS
    // 'ref' means that WRITER is a reference to a constant value, and this reference is immutable but the data it points to may be mutable, as in this case)
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0, // We start writing in the first column
        color_code: ColorCode::new(Color::White, Color::Black), // Sets the text color to yellow on a black background
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, // This gives WRITER access to the VGA text buffer at memory address 0xb8000, which is where text mode VGA buffers are located on x86 systems
        cursor_visible: true,
    });
}

// The standard color palette in VGA text mode.
#[allow(dead_code)] // This attribute is a compiler directive that tells Rust to suppress warnings about unused code. It’s likely added here because some colors may not be used yet, and this prevents the compiler from issuing a warning
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // This tells Rust to automatically generate implementations for these traits
#[repr(u8)] // Specifies that each 'Color' value is stored as an u8 (8-bit unsigned integer)
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
// Combines a foreground and background color into a single byte: the background color takes the upper 4 bits, and the foreground takes the lower 4 bits
struct ColorCode(u8);
impl ColorCode {
    // Create a new `ColorCode` with the given foreground and background colors
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`
#[repr(C)]
// Represents a character on the screen, combining an ASCII character and a ColorCode
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
// Represents the VGA text buffer itself, which is an array of ScreenChar instances
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// This struct manages writing text to the VGA buffer
pub struct Writer {
    column_position: usize, // Tracks the current position within a line (or column) on the screen
    color_code: ColorCode, // Stores the color in which characters will be printed
    buffer: &'static mut Buffer, // A mutable reference to a Buffer that has a static lifetime. This reference points to the entire VGA text buffer in memory
    pub cursor_visible: bool,
}

// This block defines the methods that handle writing operations for the Writer struct
impl Writer {
    // Function to show the cursor
    pub fn show_cursor(&mut self) {
        if self.column_position < BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
                ascii_character: b'_',
                color_code: self.color_code,
            });
        }
    }

    // Function to hide the cursor
    pub fn hide_cursor(&mut self) {
        if self.column_position < BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
                ascii_character: b' ', // Empty space
                color_code: self.color_code,
            });
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => {
                // Hide cursor before deleting
                self.hide_cursor();
    
                let prompt_length = self.prompt_length();
                if self.column_position > prompt_length {
                    self.column_position -= 1;
                    self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
                        ascii_character: b' ', // Change with a space character
                        color_code: self.color_code,
                    });
                }
    
                // Show the updated cursor
                self.show_cursor();
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
    
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
    
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
    
                // Show the updated cursor
                self.show_cursor();
            }
        }
    }    

    // Calculate the length of the current prompt (includes the folder path and "> ")
    fn prompt_length(&self) -> usize {
        // Suppose `self.get_prompt()` returns the prompt string
        // For example: "T> " or "exampleoffolder> "
        self.get_prompt().len()
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Handling the backspace character
                0x08 => {
                    // Hide cursor before deleting
                    self.hide_cursor();

                    let prompt_length = self.prompt_length();
                    if self.column_position > prompt_length {
                        self.column_position -= 1;
                        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
                            ascii_character: b' ',
                            color_code: self.color_code,
                        });
                    }

                    // Re-show cursor after deletion
                    self.show_cursor();
                },
                // Printable ASCII character or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Unrecognized characters
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn get_prompt(&self) -> String {
        format!("> ")
    }  

    pub fn new_line(&mut self) {
        // Hide the cursor before going to the next new line
        self.hide_cursor();
    
        // Move all rows up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    
        // Don't write the prompt here, we'll only do it when the user presses enter
        self.show_cursor(); // Show cursor again
    } 

    // Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// This implementation makes Writer compatible with Rust’s fmt::Write trait, which allows the Writer to use Rust’s formatted string methods like write_fmt
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Like the `print!` macro in the standard library, but prints to the VGA text buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

// Like the `println!` macro in the standard library, but prints to the VGA text buffer
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}