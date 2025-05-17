use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;
use crate::interrupts::INPUT_QUEUE;
use spin::MutexGuard;
use alloc::collections::vec_deque::VecDeque;


#[cfg(feature = "qwerty")]
lazy_static! {
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
        layouts::Us104Key,
        ScancodeSet1,
        HandleControl::MapLettersToUnicode
    ));
}

pub fn init() {
    crate::interrupts::set_irq_handler(1, interrupt_handler);
}

fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

fn interrupt_handler() {
    let mut keyboard = KEYBOARD.lock();
    let scancode = read_scancode();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let c = match key {
                DecodedKey::Unicode(c)    => c,
                DecodedKey::RawKey(kc)    => match kc {
                    KeyCode::ArrowLeft  => '←',
                    KeyCode::ArrowUp    => '↑',
                    KeyCode::ArrowRight => '→',
                    KeyCode::ArrowDown  => '↓',
                    _                   => return,
                }
            };
            // Instead of writing to the console, I put the char in the input queue:
            let mut q = INPUT_QUEUE.lock();
            q.push_back(c as u8);
        }
    }
}