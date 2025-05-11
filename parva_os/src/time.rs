// Import atomic types and spin loop hint from the core crate.
use core::sync::atomic::{spin_loop_hint, AtomicU64, Ordering};

// A global atomic variable representing how many CPU cycles happen per nanosecond.
// This must be initialized at runtime (e.g. via calibration) for nanowait() to be accurate.
static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);

// Reads the current value of the CPU's timestamp counter using the RDTSC instruction.
// This value increases steadily with the number of CPU cycles since reset.
fn rdtsc() -> u64 {
    unsafe {
        // Ensures all previous instructions have completed before reading the timestamp counter.
        core::arch::x86_64::_mm_lfence();

        // Read the time-stamp counter, which returns the number of CPU cycles since startup.
        core::arch::x86_64::_rdtsc()
    }
}

// Busy-waits for a specified number of nanoseconds using the timestamp counter.
pub fn nanowait(nanoseconds: u64) {
    // Record the current timestamp counter value.
    let start = rdtsc();

    // Compute how many CPU cycles to wait based on the number of nanoseconds requested.
    let delta = nanoseconds * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);

    // Continuously check the elapsed CPU cycles until the required delay is reached.
    while rdtsc() - start < delta {
        // Hint to the CPU that we are in a spin-wait loop to reduce power consumption
        // and improve performance on hyper-threaded systems.
        spin_loop_hint();
    }
}