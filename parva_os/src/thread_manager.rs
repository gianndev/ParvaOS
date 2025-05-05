use core::ptr;
use alloc::vec::Vec;

struct Thread {
    stack: usize,
    program_counter: usize,
    registers: [usize; 16], // assuming 16 registers (change as needed)
    state: ThreadState, // Add a field to track the thread's state
    id: usize, // Add a unique ID for each thread
}

enum ThreadState {
    Running,
    Sleeping,
    Waiting,
    Zombie,
}

struct Scheduler {
    threads: Vec<Thread>,
    current_thread_id: usize,
}

impl Scheduler {
    fn add_thread(&mut self, mut thread: Thread) {
        thread.id = self.threads.len();
        self.threads.push(thread);
    }

    fn current_thread(&self) -> &Thread {
        &self.threads[self.current_thread_id]
    }

    fn current_thread_mut(&mut self) -> &mut Thread {
        &mut self.threads[self.current_thread_id]
    }

    fn switch_to(&mut self, thread_id: usize) {
        let current_thread = self.current_thread_mut();
        current_thread.program_counter = Scheduler::get_current_instruction_pointer();

        self.current_thread_id = thread_id;
        let new_thread = self.current_thread_mut();
        Scheduler::set_instruction_pointer(new_thread.program_counter);
    }

    fn schedule(&mut self) {
        let next_thread_id = (self.current_thread_id + 1) % self.threads.len();
        self.switch_to(next_thread_id);
    }

    fn get_current_instruction_pointer() -> usize {
        0 // Placeholder
    }

    fn set_instruction_pointer(_addr: usize) {
        // Placeholder
    }
}
