use crate::{gdt, println, hlt_loop, vga::WRITER};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use alloc::string::String;
use alloc::{format, vec::Vec};

static mut INPUT_BUFFER: String = String::new();

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        // Declare access to CURSOR_TICKS safe with `unsafe`
        unsafe {
            static mut CURSOR_TICKS: usize = 0;
            let mut writer = WRITER.lock();

            CURSOR_TICKS += 1;
            if CURSOR_TICKS % 10 == 0 { // Flash every 10 timer ticks
                if writer.cursor_visible {
                    writer.hide_cursor();
                } else {
                    writer.show_cursor();
                }
                writer.cursor_visible = !writer.cursor_visible;
            }
        }
    });

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    if character == '\n' {
                        let mut writer = WRITER.lock();
                        writer.new_line();
                        unsafe {
                            // Process the command written by the user
                            process_command(&INPUT_BUFFER, &mut writer);

                            // Clears the buffer for the next input
                            INPUT_BUFFER.clear();

                            // Show the prompt
                            writer.write_string(format!("> ").as_str());
                        }
                    } else if character == '\x08' {  // \x08 is the ASCII code for Backspace
                        unsafe {
                            let mut writer = WRITER.lock();
                
                            if !INPUT_BUFFER.is_empty() {
                                // Remove the last character from the buffer
                                INPUT_BUFFER.pop();
                
                                // Clear the last character on the screen (backspace behavior)
                                writer.write_byte(0x08);  // ASCII value for backspace
                                writer.write_byte(b' ');   // Overwrite the character with a space
                                writer.write_byte(0x08);  // Move the cursor back again
                            }
                        }
                    } else {
                        unsafe {
                            let mut writer = WRITER.lock();

                            // Add the character to the input buffer
                            INPUT_BUFFER.push(character);

                            // Show the character on the screen
                            writer.write_byte(character as u8);
                        }
                    }
                },
                DecodedKey::RawKey(_key) => {},
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

fn process_command(command: &str, writer: &mut crate::vga::Writer) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    match parts.as_slice() {
        // 'hello' command
        ["hello"] => {
            writer.write_string("Hello World!\n");
        }
        // Empty Command
        [] => {
            // Ignore empty command
        }
        // Unknown command
        _ => {
            writer.write_string("Unknown Command\n");
        }
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}