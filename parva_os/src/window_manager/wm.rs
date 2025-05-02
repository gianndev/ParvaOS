use alloc::{borrow::ToOwned, string::String, vec::Vec, vec};

use crate::{time::sleep, vga::{Color, ColorCode, ScreenChar, BUFFER_HEIGHT, BUFFER_WIDTH}};

const DESKTOP_BG: Color = Color::LightBlue; // Define the background color for the desktop

type Buffer2D = [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT];

fn background() -> Buffer2D {
    // Define a blank character with white foreground and blue background
    let blank = ScreenChar {
        ascii_character: b' ',
        color_code: ColorCode::new(Color::White, DESKTOP_BG),
    };

    // Fill the entire screen with the blank character
    let buf = [[blank; BUFFER_WIDTH]; BUFFER_HEIGHT];

    buf
}

pub struct Window {
    contents: Vec<Vec<ScreenChar>>,
    name: String,
    x_pos: usize,
    y_pos: usize,
    width: usize,
    height: usize,
}

impl Window {
    pub fn new(name: String, x_pos: usize, y_pos: usize, width: usize, height: usize) -> Self {
        let contents = vec![vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::LightGray)); width]; height];
        Self {
            contents,
            name,
            x_pos,
            y_pos,
            width,
            height,
        }
    }

    pub fn draw(&self, buffer: &mut Buffer2D) {
        for row in 0..self.height {
            for col in 0..self.width {
                buffer[self.y_pos + row][self.x_pos + col] = self.contents[row][col];
            }
        }
    }
}

pub struct Desktop {
    buffer: &'static mut Buffer2D,
}

impl Desktop {
    pub fn new() -> Self {
        // Map to VGA buffer in memory
        let buffer = unsafe { &mut *(0xb8000 as *mut Buffer2D) };
        buffer.copy_from_slice(&background());
        Self { buffer }
    }

    pub fn display(&mut self) {
        // Loop through the entire screen
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // Always display a space with white-on-blue
                self.buffer[row][col].ascii_character = b' ';
                self.buffer[row][col].color_code = ColorCode::new(Color::White, DESKTOP_BG);
            }
        }
    }
}

pub fn gui() -> ! {
    let window1 = Window::new("Welcome to Parva OS".to_owned(), 10, 5, 40, 10);
    let mut desktop = Desktop::new();

    loop {
        desktop.display();
        window1.draw(desktop.buffer);
        sleep(10_000_000);
    }
    
}
