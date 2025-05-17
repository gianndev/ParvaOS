use core::sync::atomic::{spin_loop_hint, AtomicUsize, AtomicU64, Ordering};
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

// CMOS RTC register addresses
#[repr(u8)]
enum RtcRegister {
    Second  = 0x00,
    Minute  = 0x02,
    Hour    = 0x04,
    Day     = 0x07,
    Month   = 0x08,
    Year    = 0x09,
    StatusA = 0x0A,
    StatusB = 0x0B,
    StatusC = 0x0C,
}

// Simple struct to hold a full RTC timestamp
#[derive(Debug)]
pub struct RtcTime {
    pub year:   u16,
    pub month:  u8,
    pub day:    u8,
    pub hour:   u8,
    pub minute: u8,
    pub second: u8,
}

// Programmable Interval Timer constants
const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0;  // ~1.193 MHz input clock / 3
const PIT_DIVIDER:   usize = 1193;             // divisor for ~1 ms tick
const PIT_INTERVAL:  f64   = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

static PIT_TICKS:              AtomicUsize = AtomicUsize::new(0);
static LAST_RTC_UPDATE:        AtomicUsize = AtomicUsize::new(0);
static CLOCKS_PER_NANOSECOND:  AtomicU64   = AtomicU64::new(0);

// Returns the number of PIT ticks since boot.
pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

// Returns the interval in seconds between two PIT ticks.
pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

// Returns the tick index at which the last RTC interrupt occurred.
pub fn last_rtc_update() -> usize {
    LAST_RTC_UPDATE.load(Ordering::Relaxed)
}

// Execute the `hlt` instruction to sleep until the next interrupt.
pub fn halt() {
    hlt();
}

// Read the CPU’s timestamp counter.
fn rdtsc() -> u64 {
    unsafe {
        // Ensure all prior instructions complete before reading TSC
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc()
    }
}

// Read the current date & time from the CMOS real-time clock.
pub fn read_rtc() -> RtcTime {
    // Use 8-bit I/O ports for RTC address and data
    let mut addr: Port<u8> = Port::new(0x70);
    let mut data: Port<u8> = Port::new(0x71);
    
    // Enable 24-hour mode in StatusB: set bit 1
    unsafe {
        addr.write(RtcRegister::StatusB as u8 | 0x80);
        let prev = data.read();
        addr.write(RtcRegister::StatusB as u8 | 0x80);
        data.write(prev | 0x02);
    }

    // Wait while Update-in-Progress flag (bit 7 of StatusA) is set
    while unsafe {
        addr.write(RtcRegister::StatusA as u8 | 0x80);
        data.read() & 0x80 != 0
    } {
        hlt();
    }

    // Closure to read an RTC register (must be mutable)
    let mut read = |reg: RtcRegister| -> u8 {
        unsafe {
            addr.write(reg as u8 | 0x80);
            data.read()
        }
    };

    // Populate the RtcTime struct with raw register values
    let mut time = RtcTime {
        second: read(RtcRegister::Second),
        minute: read(RtcRegister::Minute),
        hour:   read(RtcRegister::Hour),
        day:    read(RtcRegister::Day),
        month:  read(RtcRegister::Month),
        year:   read(RtcRegister::Year) as u16,
    };

    // If data is in BCD format (StatusB bit 2 = 0), convert to binary
    let status_b = read(RtcRegister::StatusB);
    if status_b & 0x04 == 0 {
        let bcd_to_bin = |n: u8| (n & 0x0F) + ((n >> 4) * 10);
        time.second = bcd_to_bin(time.second);
        time.minute = bcd_to_bin(time.minute);
        time.hour   = bcd_to_bin(time.hour & 0x7F);
        time.day    = bcd_to_bin(time.day);
        time.month  = bcd_to_bin(time.month);
        time.year   = bcd_to_bin(time.year as u8) as u16;
    }

    // CMOS year is stored as year - 2000
    time.year += 2000;
    time
}

// Returns true if the given year is a leap year.
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// Compute days since Unix epoch (1970-01-01).
fn days_since_epoch(year: u16, month: u8, day: u8) -> u64 {
    let mut days = 0u64;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    // Days in each month (non-leap)
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += month_days[m as usize - 1] as u64;
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }
    days + day as u64 - 1
}

// Returns the approximate uptime in seconds (fractional).
pub fn uptime() -> f64 {
    ticks() as f64 * time_between_ticks()
}

// Returns the current real time as a Unix timestamp (seconds.fraction).
pub fn realtime() -> f64 {
    let t = read_rtc();
    let secs = days_since_epoch(t.year, t.month, t.day) * 86400
        + (t.hour   as u64) * 3600
        + (t.minute as u64) * 60
        + (t.second as u64);

    // Manually compute fractional part of uptime
    let up = uptime();
    let frac = up - (up as u64) as f64;
    secs as f64 + frac
}

// PIT interrupt handler: increments the global tick counter.
pub fn pit_interrupt_handler() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

// RTC interrupt handler: records last update tick and clears StatusC.
pub fn rtc_interrupt_handler() {
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
    unsafe {
        let mut addr: Port<u8> = Port::new(0x70);
        let mut data: Port<u8> = Port::new(0x71);
        addr.write(RtcRegister::StatusC as u8 | 0x80);
        data.read(); // Dummy read to acknowledge interrupt
    }
}

// Initialize PIT and calibrate the CPU’s TSC against it.
pub fn init() {
    // Program PIT for periodic interrupts
    let divider = PIT_DIVIDER.min(65535) as u16;
    interrupts::without_interrupts(|| {
        let mut cmd:  Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40);
        unsafe {
            cmd.write(0x36u8);           // Channel 0, lobyte/hibyte, mode 3
            data.write(divider as u8);   // Low byte
            data.write((divider >> 8) as u8); // High byte
        }
    });

    // Calibrate TSC: measure cycles in 0.25 seconds of uptime
    let calibration_time = 250_000; // ticks, since time_between_ticks ≈ 1ms
    let a = rdtsc();
    sleep(0.25);
    let b = rdtsc();
    CLOCKS_PER_NANOSECOND.store((b - a) / calibration_time, Ordering::Relaxed);
}

// Busy-wait sleep using HLT to save cycles.
pub fn sleep(seconds: f64) {
    let start = uptime();
    while uptime() - start < seconds {
        hlt();
    }
}

// Wait approximately `nanoseconds` using the TSC.
pub fn nanowait(nanoseconds: u64) {
    let start = rdtsc();
    let delta = nanoseconds * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);
    while rdtsc() - start < delta {
        spin_loop_hint();
    }
}