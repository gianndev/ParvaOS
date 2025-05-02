pub fn sleep(cycles: u64) {
    for _ in 0..cycles {
        core::hint::spin_loop();
    }
}
