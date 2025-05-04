use alloc::{borrow::ToOwned, string::String, vec::Vec, vec};
use crate::{time::sleep, vga::{Color, ColorCode, ScreenChar, BUFFER_HEIGHT, BUFFER_WIDTH}, interrupts::INPUT_QUEUE};

const DESKTOP_BG: Color = Color::LightBlue;

type Buffer2D = [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT];

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
    move_mode: bool,
    prev_x: usize,
    prev_y: usize,
}

impl Window {
    pub fn new(name: String, x_pos: usize, y_pos: usize, width: usize, height: usize) -> Self {
        let mut contents = vec![
            vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black)); width];
            height - 1
        ];
        
        // Add initial prompt
        let prompt = b"> ";
        for (i, &ch) in prompt.iter().enumerate() {
            contents[0][i] = ScreenChar::new(ch, ColorCode::new(Color::White, Color::Black));
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
            move_mode: false,
            prev_x: x_pos,
            prev_y: y_pos,
        }
    }

    pub fn draw(&self, buffer: &mut Buffer2D) {
        // Clear previous position if moved
        if self.x_pos != self.prev_x || self.y_pos != self.prev_y {
            self.clear_previous_position(buffer);
        }

        // Clear only the previous cursor position
        self.clear_previous_cursor(buffer);

        // Draw header (only if needed)
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

        // Draw window contents
        for (row_idx, row) in self.contents.iter().enumerate() {
            let screen_row = self.y_pos + 1 + row_idx;
            for (col_idx, &ch) in row.iter().enumerate() {
                let screen_col = self.x_pos + col_idx;
                if screen_row < BUFFER_HEIGHT && screen_col < BUFFER_WIDTH {
                    buffer[screen_row][screen_col] = ch;
                }
            }
        }

        // Draw new cursor
        let cursor_row = self.y_pos + 1 + self.current_line;
        let cursor_col = self.x_pos + self.cursor_pos;
        if cursor_row < BUFFER_HEIGHT && cursor_col < BUFFER_WIDTH {
            buffer[cursor_row][cursor_col] = ScreenChar::new(
                b'_',
                ColorCode::new(Color::White, Color::Black)
            );
        }
    }

    fn clear_previous_cursor(&self, buffer: &mut Buffer2D) {
        let prev_cursor_row = self.y_pos + 1 + self.current_line;
        let prev_cursor_col = self.x_pos + self.cursor_pos;
        if prev_cursor_row < BUFFER_HEIGHT && prev_cursor_col < BUFFER_WIDTH {
            buffer[prev_cursor_row][prev_cursor_col] = ScreenChar::new(
                self.contents[self.current_line][self.cursor_pos].ascii_character,
                ColorCode::new(Color::White, Color::Black)
            );
        }
    } 

    pub fn move_window(&mut self, dx: isize, dy: isize) {
        self.prev_x = self.x_pos;
        self.prev_y = self.y_pos;
        
        // Calculate new position with bounds checking
        let new_x = (self.x_pos as isize + dx)
            .max(0)
            .min((BUFFER_WIDTH - self.width) as isize) as usize;
            
        let new_y = (self.y_pos as isize + dy)
            .max(0)
            .min((BUFFER_HEIGHT - self.height - 1) as isize) as usize;

        if new_x != self.x_pos || new_y != self.y_pos {
            self.x_pos = new_x;
            self.y_pos = new_y;
            self.needs_redraw = true;
        }
    }

    fn clear_previous_position(&self, buffer: &mut Buffer2D) {
        // Clear previous header
        for col in 0..self.width {
            buffer[self.prev_y][self.prev_x + col] = ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::White, DESKTOP_BG),
            };
        }
        
        // Clear previous content
        for row in 0..self.height {
            for col in 0..self.width {
                let screen_row = self.prev_y + 1 + row;
                let screen_col = self.prev_x + col;
                if screen_row < BUFFER_HEIGHT && screen_col < BUFFER_WIDTH {
                    buffer[screen_row][screen_col] = ScreenChar {
                        ascii_character: b' ',
                        color_code: ColorCode::new(Color::White, DESKTOP_BG),
                    };
                }
            }
        }
    }
}

pub struct Desktop {
    buffer: &'static mut Buffer2D,
    needs_initial_draw: bool,
}

impl Desktop {
    pub fn new() -> Self {
        let buffer = unsafe { &mut *(0xb8000 as *mut Buffer2D) };
        let mut desktop = Self {
            buffer,
            needs_initial_draw: true,
        };
        desktop.initialize_background();
        desktop
    }

    fn initialize_background(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer[row][col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::White, DESKTOP_BG),
                };
            }
        }
        self.needs_initial_draw = false;
    }

    pub fn display(&mut self) {
        // Only used for initial draw
        if self.needs_initial_draw {
            self.initialize_background();
        }
    }
}

pub fn gui() -> ! {
    let mut window1 = Window::new("Terminal".to_owned(), 10, 5, 50, 15);
    let mut desktop = Desktop::new();
    let mut needs_redraw = true;

    // Initial draw
    desktop.display();
    window1.draw(desktop.buffer);

    loop {
        let mut queue = INPUT_QUEUE.lock();
        let had_input = !queue.is_empty();
        while let Some(ch) = queue.pop_front() {
            handle_input(&mut window1, ch);
        }
        drop(queue);

        if had_input || needs_redraw {
            // Only redraw desktop background when exiting move mode
            if !window1.move_mode && window1.prev_x != window1.x_pos || window1.prev_y != window1.y_pos {
                desktop.display();
            }
            
            window1.draw(desktop.buffer);
            needs_redraw = false;
        }

        sleep(10_000);
        
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            if COUNTER % 50_000 == 0 {
                needs_redraw = true;
                COUNTER = 0;
            }
        }
    }
}

fn handle_input(window: &mut Window, ch: u8) {
    if window.move_mode {
        match ch {
            0x1B => { // Escape key
                window.move_mode = false;
                return;
            },
            b'w' => window.move_window(0, -1),
            b's' => window.move_window(0, 1),
            b'a' => window.move_window(-1, 0),
            b'd' => window.move_window(1, 0),
            _ => {},
        }
        return;
    }

    match ch {
        b'\n' => {
            // Process command
            let command = window.input_buffer.clone();
            window.command_history.push(command.clone());
            
            // Add output line
            let response = if command == "hello" {
                "Hello World!"
            } else if command == "info" {
                "ParvaOS version 0.0.2"
            } else if command == "help" {
                "hello | prints hello world\nhelp  | list of all commands\ninfo  | shows OS version"
            } else if !command.is_empty() {
                "Unknown command"
            } else {
                ""
            };

            // Process response with potential newlines
            if !response.is_empty() {
                for line in response.split('\n') {
                    add_output_line(window, line);
                }
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
                    ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black));
            }
        },
        0x09 => { // Tab key
            window.move_mode = true;
            return;
        },
        _ => {
            // Allow space (0x20) and all printable ASCII characters
            if window.cursor_pos < window.width && (ch == b' ' || ch.is_ascii_graphic()) {
                window.input_buffer.push(ch as char);
                window.contents[window.current_line][window.cursor_pos] = 
                    ScreenChar::new(ch, ColorCode::new(Color::White, Color::Black));
                window.cursor_pos += 1;
            }
        }
    }

    window.needs_redraw = true;
}

fn add_new_line(window: &mut Window) {
    window.needs_redraw = true;
    window.current_line += 1;
    if window.current_line >= window.height - 1 {
        // Scroll up
        window.contents.remove(0);
        window.contents.push(vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black)); window.width]);
        window.current_line = window.height - 2;
    }
    
    // Add new prompt
    let prompt = b"> ";
    for (i, &ch) in prompt.iter().enumerate() {
        window.contents[window.current_line][i] = 
            ScreenChar::new(ch, ColorCode::new(Color::White, Color::Black));
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
        window.contents.push(vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black)); window.width]);
        window.current_line = window.height - 2;
    }

    // Add output without prompt
    for (i, &ch) in bytes.iter().take(max_len).enumerate() {
        window.contents[window.current_line][i] = 
            ScreenChar::new(ch, ColorCode::new(Color::White, Color::Black));
    }
}