use crate::vga::{ScreenChar, ColorCode, Color, BUFFER_WIDTH, BUFFER_HEIGHT};

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
    let mut desktop = Desktop::new();
    loop {
        desktop.display();
    }
}
