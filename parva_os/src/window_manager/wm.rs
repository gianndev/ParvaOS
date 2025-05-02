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
        let contents = vec![
            vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::LightGray)); width];
            height - 1
        ];
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
        // 1. Draw header row
        let header_color = ColorCode::new(Color::White, Color::Blue);
        let header_row = self.y_pos;
        for col in 0..self.width {
            buffer[header_row][self.x_pos + col] = ScreenChar::new(b' ', header_color);
        }
    
        // Write the name centered in the header
        let name_bytes = self.name.as_bytes();
        let start = (self.width.saturating_sub(name_bytes.len())) / 2;
        for (i, &b) in name_bytes.iter().enumerate() {
            if start + i < self.width {
                buffer[header_row][self.x_pos + start + i] = ScreenChar::new(b, header_color);
            }
        }
    
        // 2. Draw content rows (starting from next line)
        for row in 0..self.contents.len() {
            for col in 0..self.width {
                buffer[self.y_pos + 1 + row][self.x_pos + col] = self.contents[row][col];
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
    let mut window1 = Window::new(
        "Welcome".to_owned(),
        10,
        5,
        50,
        15,
    );

    // Write a welcome message in the first row of the window
    let text = b"Welcome to ParvaOS";
    let text_color = ColorCode::new(Color::Black, Color::LightGray);
    let start_col = (window1.width.saturating_sub(text.len())) / 2;

    for (i, &ch) in text.iter().enumerate() {
        if start_col + i < window1.width {
            window1.contents[0][start_col + i] = ScreenChar::new(ch, text_color);
        }
    }

    let mut desktop = Desktop::new();

    loop {
        desktop.display();
        window1.draw(desktop.buffer);
        sleep(10_000_000);
    }
}