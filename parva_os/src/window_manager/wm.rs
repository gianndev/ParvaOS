use alloc::{borrow::ToOwned, string::String, vec::Vec, vec};
use crate::interrupts::INPUT_QUEUE;
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
    input_buffer: String,
    command_history: Vec<String>,
    current_line: usize,
    cursor_pos: usize,
    needs_redraw: bool,
}

impl Window {
    pub fn new(name: String, x_pos: usize, y_pos: usize, width: usize, height: usize) -> Self {
        let mut contents = vec![
            vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::LightGray)); width];
            height - 1
        ];
        
        // Add initial prompt
        let prompt = b"> ";
        for (i, &ch) in prompt.iter().enumerate() {
            contents[0][i] = ScreenChar::new(ch, ColorCode::new(Color::White, Color::LightGray));
        }

        Self {
            contents,
            name,
            x_pos,
            y_pos,
            width,
            height,
            input_buffer: String::new(),
            command_history: Vec::new(),
            current_line: 0,
            cursor_pos: 2,  // Start after "> "
            needs_redraw: true,
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

        for (row_idx, row) in self.contents.iter().enumerate() {
            let screen_row = self.y_pos + 1 + row_idx;
            for (col_idx, &ch) in row.iter().enumerate() {
                let screen_col = self.x_pos + col_idx;
                if screen_row < BUFFER_HEIGHT && screen_col < BUFFER_WIDTH {
                    buffer[screen_row][screen_col] = ch;
                }
            }
        }
    
        // Draw cursor
        let cursor_row = self.y_pos + 1 + self.current_line;
        let cursor_col = self.x_pos + self.cursor_pos;
        if cursor_row < BUFFER_HEIGHT && cursor_col < BUFFER_WIDTH {
            buffer[cursor_row][cursor_col] = ScreenChar::new(b'_', 
                ColorCode::new(Color::White, Color::LightGray));
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
    let mut window1 = Window::new("Terminal".to_owned(), 10, 5, 50, 15);
    let mut desktop = Desktop::new();
    let mut needs_redraw = true;

    loop {
        // Process all pending input first
        let mut queue = INPUT_QUEUE.lock();
        let had_input = !queue.is_empty();
        while let Some(ch) = queue.pop_front() {
            handle_input(&mut window1, ch);
        }
        drop(queue);

        // Only redraw if we had input or periodically for cursor blink
        if had_input || needs_redraw {
            desktop.display();
            window1.draw(desktop.buffer);
            needs_redraw = false;
        }

        // Shorter sleep but maintain cursor blink timing
        sleep(10_000);
        
        // Force periodic redraw for cursor blink (every 500ms)
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            if COUNTER % 50_000 == 0 { // 50,000 * 10μs = 500ms
                needs_redraw = true;
                COUNTER = 0;
            }
        }
    }
}

fn handle_input(window: &mut Window, ch: u8) {
    window.needs_redraw = true;
    match ch {
        b'\n' => {
            // Process command
            let command = window.input_buffer.clone();
            window.command_history.push(command.clone());
            
            // Add output line
            let response = if command == "hello" {
                "Hello World!"
            } else if !command.is_empty() {
                "Unknown command"
            } else {
                ""
            };

            // Add output line FIRST
            if !response.is_empty() {
                add_output_line(window, response);
            }

            // THEN add new prompt line
            add_new_line(window);
            window.input_buffer.clear();
            window.cursor_pos = 2;
        },
        0x08 => { // Backspace
            if window.cursor_pos > 2 && !window.input_buffer.is_empty() {
                window.input_buffer.pop();
                window.cursor_pos -= 1;
                window.contents[window.current_line][window.cursor_pos] = 
                    ScreenChar::new(b' ', ColorCode::new(Color::White, Color::LightGray));
            }
        },
        _ => {
            // Allow space (0x20) and all printable ASCII characters
            if window.cursor_pos < window.width && (ch == b' ' || ch.is_ascii_graphic()) {
                window.input_buffer.push(ch as char);
                window.contents[window.current_line][window.cursor_pos] = 
                    ScreenChar::new(ch, ColorCode::new(Color::White, Color::LightGray));
                window.cursor_pos += 1;
            }
        }
    }
}

fn add_new_line(window: &mut Window) {
    window.needs_redraw = true;
    window.current_line += 1;
    if window.current_line >= window.height - 1 {
        // Scroll up
        window.contents.remove(0);
        window.contents.push(vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::LightGray)); window.width]);
        window.current_line = window.height - 2;
    }
    
    // Add new prompt
    let prompt = b"> ";
    for (i, &ch) in prompt.iter().enumerate() {
        window.contents[window.current_line][i] = 
            ScreenChar::new(ch, ColorCode::new(Color::White, Color::LightGray));
    }
}

fn add_output_line(window: &mut Window, text: &str) {
    window.needs_redraw = true;
    
    let bytes = text.as_bytes();
    let max_len = window.width.min(bytes.len());
    
    window.current_line += 1;
    if window.current_line >= window.height - 1 {
        // Scroll up both contents and maintain current_line position
        window.contents.remove(0);
        window.contents.push(vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::LightGray)); window.width]);
        window.current_line = window.height - 2;
    }

    // Add output without prompt
    for (i, &ch) in bytes.iter().take(max_len).enumerate() {
        window.contents[window.current_line][i] = 
            ScreenChar::new(ch, ColorCode::new(Color::White, Color::LightGray));
    }
}