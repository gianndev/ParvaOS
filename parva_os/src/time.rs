pub fn sleep(cycles: u64) {
    unsafe {
        let start = x86_64::instructions::port::Port::<u32>::new(0x40).read();
        while (x86_64::instructions::port::Port::<u32>::new(0x40).read().wrapping_sub(start) as u64) < cycles {}
    }
}