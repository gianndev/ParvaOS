// Import atomic types and spin loop hint from the core crate.
use core::sync::atomic::{spin_loop_hint, AtomicU64, AtomicUsize, Ordering};

// PIT (Programmable Interval Timer) frequency in Hz.
// This is derived from the standard PIT input clock (3.579545 MHz / 3).
const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0; // ≈ 1,193,181.666 Hz

// Divider used to set the PIT frequency.
// A divider of 1193 gives an interrupt rate of roughly 1000 Hz (1 ms interval).
const PIT_DIVIDER: usize = 1193;

// Duration in seconds of one PIT tick, computed from the divider and the frequency.
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

// Atomic counter for the number of PIT ticks that have occurred since boot.
// Incremented by the PIT interrupt handler elsewhere in the kernel.
static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);

// Number of CPU clock cycles that occur per nanosecond.
// This should be calibrated at runtime using both PIT and RDTSC for accurate timing.
static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);

// Reads the current value of the CPU's time-stamp counter using the `rdtsc` instruction.
//
// This counter is a 64-bit register that increments with every CPU cycle.
// It can be used for high-resolution timing and profiling.
fn rdtsc() -> u64 {
    unsafe {
        // Ensure instruction ordering: wait until all prior instructions are completed.
        core::arch::x86_64::_mm_lfence();

        // Read and return the current value of the time-stamp counter.
        core::arch::x86_64::_rdtsc()
    }
}

// Busy-waits for a specified number of nanoseconds using the time-stamp counter.
//
// This function uses a spin loop and is extremely precise, suitable for microsecond or nanosecond delays.
// ⚠ Note: It blocks the CPU and should not be used for long delays in multitasking environments.
pub fn nanowait(nanoseconds: u64) {
    // Get the starting value of the timestamp counter.
    let start = rdtsc();

    // Compute the number of CPU cycles corresponding to the desired delay.
    // This uses the globally calibrated clocks-per-nanosecond value.
    let delta = nanoseconds * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);

    // Spin in a loop until the difference in CPU cycles reaches the target.
    while rdtsc() - start < delta {
        // This hint tells the CPU we're in a busy-wait loop and can reduce power consumption
        // or improve performance on modern CPUs with hyper-threading.
        spin_loop_hint();
    }
}

// Returns the number of PIT ticks since the system booted.
//
// Each tick corresponds to a fixed interval, typically 1 ms if the PIT is configured with a divider of 1193.
pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

// Returns the time duration of a single PIT tick in seconds.
//
// Useful for converting PIT tick counts into wall-clock time.
pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

// Returns the system uptime in seconds since boot.
//
// This is calculated as the number of PIT ticks multiplied by the interval of each tick.
pub fn uptime() -> f64 {
    time_between_ticks() * ticks() as f64
}
