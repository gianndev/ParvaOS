use alloc::{borrow::ToOwned, string::String, vec::Vec, vec};
use x86_64::instructions::hlt;
use crate::{vga::{Color, ColorCode, ScreenChar, BUFFER_HEIGHT, BUFFER_WIDTH}, interrupts::INPUT_QUEUE};

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
    is_fullscreen: bool,       
    original_x: usize,         
    original_y: usize,         
    original_width: usize,     
    original_height: usize,    
    needs_desktop_redraw: bool,
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
            is_fullscreen: false,
            original_x: x_pos,
            original_y: y_pos,
            original_width: width,
            original_height: height,
            needs_desktop_redraw: false,
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
        // Clear previous header with bounds checking
        for col in 0..self.width {
            let screen_col = self.prev_x + col;
            if self.prev_y < BUFFER_HEIGHT && screen_col < BUFFER_WIDTH {
                buffer[self.prev_y][screen_col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::White, DESKTOP_BG),
                };
            }
        }
        
        // Clear previous content with bounds checking
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
    back_buffer: Buffer2D,
    vga_buffer: &'static mut Buffer2D,
    needs_initial_draw: bool,
}

impl Desktop {
    pub fn new() -> Self {
        // VGA text-mode starts at 0xb8000
        let vga_buffer = unsafe { &mut *(0xb8000 as *mut Buffer2D) };
        // initialize RAM back buffer to spaces
        let back_buffer = [[
            ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::White, DESKTOP_BG),
            };
            BUFFER_WIDTH
        ]; BUFFER_HEIGHT];

        let mut d = Self {
            back_buffer,
            vga_buffer,
            needs_initial_draw: true,
        };
        d.initialize_background();
        d.flush(); // paint the first full frame
        d
    }

    // Fill back_buffer with desktop background
    fn initialize_background(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.back_buffer[row][col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::White, DESKTOP_BG),
                };
            }
        }
        self.needs_initial_draw = false;
    }

    // Compare back_buffer to vga_buffer, only write changed cells
    fn flush(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let new = self.back_buffer[row][col];
                let old = self.vga_buffer[row][col];
                if new != old {
                    // only these writes actually touch VGA RAM
                    self.vga_buffer[row][col] = new;
                }
            }
        }
    }

    pub fn display(&mut self) {
        self.initialize_background();
    }
}

pub fn gui() -> ! {
    let mut window1 = Window::new("Terminal".to_owned(), 10, 5, 50, 15);
    let mut desktop = Desktop::new();

    // initial draw already done by Desktop::new()
    window1.draw(&mut desktop.back_buffer);
    desktop.flush();

    loop {
        // halt until next interrupt (keyboard or timer)
        hlt();

        let mut queue = INPUT_QUEUE.lock();
        let had_input = !queue.is_empty();
        while let Some(ch) = queue.pop_front() {
            handle_input(&mut window1, ch);
        }
        drop(queue);

        if had_input || window1.needs_desktop_redraw {
            // if desktop needs full redraw (e.g. on exit fullscreen), repaint background
            if window1.needs_desktop_redraw {
                desktop.initialize_background();
                window1.needs_desktop_redraw = false;
            }
            // if window moved, also clear old desktop area
            if !window1.move_mode
                && (window1.prev_x != window1.x_pos || window1.prev_y != window1.y_pos)
            {
                desktop.initialize_background();
            }

            // render window into back_buffer
            window1.draw(&mut desktop.back_buffer);
            // push only diffs to VGA
            desktop.flush();

            window1.needs_redraw = false;
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
            b' ' => { // Space key toggles fullscreen
                if window.is_fullscreen {
                    // Restore original size and position
                    window.x_pos = window.original_x;
                    window.y_pos = window.original_y;
                    window.width = window.original_width;
                    window.height = window.original_height;
                    window.is_fullscreen = false;
                    window.needs_desktop_redraw = true;
                    
                    // Reset contents to original size (keep last N lines)
                    let target_lines = window.height - 1;
                    let mut new_contents = vec![
                        vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black)); window.width];
                        target_lines
                    ];
                    
                    // Calculate how many lines we can copy from the end
                    let start_line = window.contents.len().saturating_sub(target_lines);
                    
                    // Copy lines while preserving prompt visibility
                    for (i, row) in window.contents.iter().skip(start_line).enumerate() {
                        let copy_len = row.len().min(window.width);
                        new_contents[i][..copy_len].copy_from_slice(&row[..copy_len]);
                        
                        // Always ensure last line has prompt
                        if i == target_lines - 1 {
                            let prompt = b"> ";
                            for (col, &ch) in prompt.iter().enumerate() {
                                if col < window.width {
                                    new_contents[i][col] = ScreenChar::new(ch, ColorCode::new(Color::White, Color::Black));
                                }
                            }
                        }
                    }
                    
                    window.contents = new_contents;
                    window.current_line = target_lines.saturating_sub(1);
                    window.cursor_pos = 2 + window.input_buffer.len().min(window.width - 2);
                } else {
                    // Save current state
                    window.original_x = window.x_pos;
                    window.original_y = window.y_pos;
                    window.original_width = window.width;
                    window.original_height = window.height;
                    
                    // Enter fullscreen
                    window.x_pos = 0;
                    window.y_pos = 0;
                    window.width = BUFFER_WIDTH;
                    window.height = BUFFER_HEIGHT;
                    window.is_fullscreen = true;
                    
                    // Expand contents while preserving history
                    let mut new_contents = vec![
                        vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black)); BUFFER_WIDTH];
                        BUFFER_HEIGHT - 1
                    ];
                    
                    // Copy existing lines to bottom of new buffer
                    let start_line = new_contents.len().saturating_sub(window.contents.len());
                    for (i, row) in window.contents.iter().enumerate() {
                        let copy_len = row.len().min(BUFFER_WIDTH);
                        new_contents[start_line + i][..copy_len].copy_from_slice(&row[..copy_len]);
                    }
                    
                    window.contents = new_contents;
                    window.current_line = BUFFER_HEIGHT - 2;  // Start at bottom
                }
                window.needs_redraw = true;
            },
            _ => {},
        }
        return;
    }

    match ch {
        b'\n' => {
            // Process command
            let command = window.input_buffer.clone();
            window.command_history.push(command.clone());
            
            let response = if command == "hello" {
                "Hello World!"
            } else if command == "clear" {
                // Reset terminal content to initial state
                window.contents = vec![
                    vec![ScreenChar::new(b' ', ColorCode::new(Color::White, Color::Black)); window.width];
                    window.height - 1
                ];
                
                // Add initial prompt
                let prompt = b"> ";
                for (i, &ch) in prompt.iter().enumerate() {
                    window.contents[0][i] = ScreenChar::new(ch, ColorCode::new(Color::White, Color::Black));
                }
                
                window.current_line = 0;
                window.cursor_pos = 2;
                window.input_buffer.clear();
                window.needs_redraw = true;
                return;
            } else if command == "shutdown" {
                crate::exit_qemu(crate::QemuExitCode::Success);
                crate::hlt_loop();
            } else if command == "reboot" {
                crate::reboot();
            } else if command == "info" {
                "ParvaOS version 0.0.2"
            } else if command == "help" {
                "clear    | clear terminal\n\
                 hello    | prints hello world\n\
                 help     | list of commands\n\
                 info     | shows OS version\n\
                 reboot   | restart system\n\
                 shutdown | power off system\n\
                 [TAB]    | enter move mode (move with WASD)\n\
                 [SPACE]  | toggle fullscreen"
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