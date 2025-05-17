// ATA command codes sent to the drive’s command register

use core::sync::atomic::spin_loop_hint;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use crate::time;
use bit_field::BitField;
use alloc::{string::String, vec::Vec};
use crate::alloc::string::ToString;
use lazy_static::lazy_static;
use spin::Mutex;

#[repr(u16)]
enum Command {
    Read = 0x20, // Read sectors: Instructs the drive to transfer one or more sectors from the disk into its data register.
    Write = 0x30, // Write sectors: Instructs the drive to transfer one or more sectors from its data register out to the disk.
    Identify = 0xEC, // Identify drive: Requests the drive to return a 512-byte block of identification data (model, serial, capabilities).
}

// Status register bits for an ATA device, read from the status register
#[allow(dead_code)]
#[repr(usize)]
enum Status {
    Error = 0,          // Error (bit 0): Indicates that the last command resulted in an Error. Check the Error register for detailed Error information.
    Index = 1,          // Index (bit 1): Historically signaled the drive head passing the index on removable media (floppy disks). Rarely used on modern hard drives.
    CorrectedData = 2,  // Corrected Data (bit 2): Indicates that a data Error was detected and automatically corrected (e.g., via ECC).
    DataRequest = 3,    // Data Request (bit 3): The drive is ready to transfer data. The host should read from or write to the data register now.
    Service = 4,        // Service (bit 4): Used in ATAPI (packet) mode to indicate the drive needs service from the host.
    DeviceFault = 5,    // Device Fault (bit 5): Indicates a non-recoverable fault in the device (hardware failure).
    DeviceReady = 6,    // Device Ready (bit 6): The device is powered up and ready to accept commands (but not necessarily ready to transfer data).
    Busy = 7,           // Busy (bit 7): The device is busy processing the current command. Until this bit clears, the host must not send new commands.
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bus {
    id: u8, // Logical identifier for the bus (e.g., 0 for primary, 1 for secondary)
    irq: u8, // IntErrorupt Request Line (IRQ) used by the device to notify the CPU (e.g., IRQ 14 or 15 for ATA)
    data_register: Port<u16>, // 16-bit data register: used to transfer sector data to/from the drive.
    error_register: PortReadOnly<u8>, // Read-only error register: contains error codes after failed operations.
    features_register: PortWriteOnly<u8>, // Write-only features register: used to send advanced command features to the drive.
    sector_count_register: Port<u8>, // Specifies the number of sectors to transfer (usually 1).
    lba0_register: Port<u8>, // 8-bit LBA (Logical Block Addressing) low byte (bits 0–7 of the sector address).
    lba1_register: Port<u8>, // LBA mid byte (bits 8–15 of the sector address).
    lba2_register: Port<u8>, // LBA high byte (bits 16–23 of the sector address).
    drive_register: Port<u8>, // Drive/head register: used to select the drive (master/slave) and bits 24–27 of LBA.
    status_register: PortReadOnly<u8>, // Read-only status register: used to check device state (e.g., busy, ready, Error).
    command_register: PortWriteOnly<u8>, // Write-only command register: send commands like Read, Write, Identify.
    alternate_status_register: PortReadOnly<u8>, // Read-only alternate status register: same as status but does not clear intErrorupt flags.
    control_register: PortWriteOnly<u8>, // Write-only control register: used to send control signals like reset.
    drive_blockess_register: PortReadOnly<u8>, // Read-only drive address register (also called Drive Address or Drive Blockless register): rarely used.
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self {
            id, irq,

            data_register: Port::new(io_base + 0),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),

            alternate_status_register: PortReadOnly::new(ctrl_base + 0),
            control_register: PortWriteOnly::new(ctrl_base + 0),
            drive_blockess_register: PortReadOnly::new(ctrl_base + 1),
        }
    }

    // Soft-reset the ATA channel by toggling the SRST bit in the control register
    fn reset(&mut self) {
        unsafe {
            self.control_register.write(4); // Set SRST = 1
            time::nanowait(5); // Wait ≥ 5 µs for reset to take effect
            self.control_register.write(0); // Clear SRST = 0
            time::nanowait(2000); // Wait ≥ 2 ms for device to reinitialize
        }
    }

    // Short delay by reading the alternate status register 4 times (~400 ns)
    fn wait(&mut self) {
        for _ in 0..4 {
            unsafe { self.alternate_status_register.read(); }
        }
    }

    // Write an ATA command code into the command register
    fn write_command(&mut self, cmd: Command) {
        unsafe {
            self.command_register.write(cmd as u8);
        }
    }

    // Read and return the current status byte from the status register
    fn status(&mut self) -> u8 {
        unsafe { self.status_register.read() }
    }

    // Read the LBA mid register (used by IDENTIFY to check for ATAPI vs ATA)
    fn lba1(&mut self) -> u8 {
        unsafe { self.lba1_register.read() }
    }

    // Read the LBA high register (used by IDENTIFY to check for ATAPI vs ATA)
    fn lba2(&mut self) -> u8 {
        unsafe { self.lba2_register.read() }
    }

    // Read one 16-bit word from the data register (sector data)
    fn read_data(&mut self) -> u16 {
        unsafe { self.data_register.read() }
    }

    // Write one 16-bit word into the data register (sector data)
    fn write_data(&mut self, data: u16) {
        unsafe { self.data_register.write(data) }
    }

    // Spin-wait until Busy clears, or time out and reset if it hangs (>1s)
    fn busy_loop(&mut self) {
        self.wait();                             // initial short delay
        let start = time::uptime();         // timestamp in seconds
        while self.is_busy() {
            if time::uptime() - start > 1.0 {
                return self.reset();             // give up and reset on hang
            }
            spin_loop_hint();                    // CPU hint for busy-wait
        }
    }

    // Check the Busy bit in the status register
    fn is_busy(&mut self) -> bool {
        self.status().get_bit(Status::Busy as usize)
    }

    // Check the Error bit in the status register
    fn is_Error(&mut self) -> bool {
        self.status().get_bit(Status::Error as usize)
    }

    // Check the DeviceReady (device ready) bit in the status register
    fn is_ready(&mut self) -> bool {
        self.status().get_bit(Status::DeviceReady as usize)
    }

    // Select the specified drive (0=master, 1=slave) on this bus
    fn select_drive(&mut self, drive: u8) {
        let drive_id = 0xA0 | (drive << 4);       // 0xA0 for master, 0xB0 for slave
        unsafe {
            self.drive_register.write(drive_id);
        }
    }

    // Uncomment to print
    // Debug helper: print the current drive and status register values
    // #[allow(dead_code)]
    // fn debug(&mut self) {
    //     self.wait();
    //     unsafe {
    //         print!("drive register: 0b{:08b}\n", self.drive_register.read());
    //         print!("status:         0b{:08b}\n", self.status_register.read());
    //     }
    // }

    // Prepare the bus to read/write one LBA block:
    // - select drive
    // - set LBA bits 0–27 (in 4 registers)
    // - set sector count = 1
    fn setup(&mut self, drive: u8, block: u32) {
        let drive_id = 0xE0 | (drive << 4);       // 0xE0 for LBA mode
        unsafe {
            // bits 24–27 of LBA go in high nibble of drive register
            self.drive_register.write(drive_id | ((block.get_bits(24..28) as u8) & 0x0F));
            self.sector_count_register.write(1);  // transfer exactly 1 sector
            self.lba0_register.write(block.get_bits(0..8) as u8);
            self.lba1_register.write(block.get_bits(8..16) as u8);
            self.lba2_register.write(block.get_bits(16..24) as u8);
        }
    }

    // IDENTIFY command: returns 256 words of device metadata if successful
    pub fn identify_drive(&mut self, drive: u8) -> Option<[u16; 256]> {
        self.reset();                            // ensure device is in known state
        self.wait();                             // short startup delay
        self.select_drive(drive);                // choose master/slave
        unsafe {
            // zero out LBA and sector count for IDENTIFY
            self.sector_count_register.write(0);
            self.lba0_register.write(0);
            self.lba1_register.write(0);
            self.lba2_register.write(0);
        }
        self.write_command(Command::Identify);   // send IDENTIFY

        if self.status() == 0 {                  // no device present?
            return None;
        }

        self.busy_loop();                        // wait until ready or reset on hang

        // if non-zero LBA registers, device is ATAPI, not ATA
        if self.lba1() != 0 || self.lba2() != 0 {
            return None;
        }

        // wait for DRQ or Error (with a max of 256 polls)
        for i in 0.. {
            if i == 256 {
                self.reset();
                return None;
            }
            if self.is_Error() {
                return None;
            }
            if self.is_ready() {
                break;
            }
        }

        // read out 256 words (512 bytes) of identify data
        let mut res = [0; 256];
        for i in 0..256 {
            res[i] = self.read_data();
        }
        Some(res)
    }

    // Read exactly one 512-byte sector from the specified drive and LBA
    pub fn read(&mut self, drive: u8, block: u32, buf: &mut [u8]) {
        assert!(buf.len() == 512);
        self.setup(drive, block);
        self.write_command(Command::Read);
        self.busy_loop();
        // read 256 words and split into bytes
        for i in 0..256 {
            let data = self.read_data();
            buf[i * 2]     = data.get_bits(0..8) as u8;
            buf[i * 2 + 1] = data.get_bits(8..16) as u8;
        }
    }

    // Write exactly one 512-byte sector to the specified drive and LBA
    pub fn write(&mut self, drive: u8, block: u32, buf: &[u8]) {
        assert!(buf.len() == 512);
        self.setup(drive, block);
        self.write_command(Command::Write);
        self.busy_loop();
        // pack bytes into 256 words and write to data register
        for i in 0..256 {
            let mut data = 0u16;
            data.set_bits(0..8, buf[i * 2] as u16);
            data.set_bits(8..16, buf[i * 2 + 1] as u16);
            self.write_data(data);
        }
        self.busy_loop();  // wait for final write completion
    }
}

// ---------- GLOBAL BUS REGISTRY ----------

lazy_static! {
    // A thread-safe vector of all ATA buses (primary, secondary, etc.)
    pub static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

// Given a count of 512-byte sectors, return (value, unit) as MB or GB.
fn disk_size(sectors: u32) -> (u32, String) {
    let bytes = sectors * 512;
    if bytes >> 20 < 1000 {
        // less than ~1000 MB → report in MB
        (bytes >> 20, String::from("MB"))
    } else {
        // otherwise report in GB
        (bytes >> 30, String::from("GB"))
    }
}

// Initialize the ATA subsystem: create primary & secondary buses, then print each drive found.
pub fn init() {
    {
        // Populate the global bus list. Standard I/O ports:
        //   primary:  0x1F0 base, 0x3F6 control, IRQ 14
        //   secondary:0x170 base, 0x376 control, IRQ 15
        let mut buses = BUSES.lock();
        buses.push(Bus::new(0, 0x1F0, 0x3F6, 14));
        buses.push(Bus::new(1, 0x170, 0x376, 15));
    }

    // Uncomment to print
    // After registering, enumerate every bus/drive pair and print model/serial/size
    // for (bus, drive, model, serial, size, unit) in list() {
    //     println!("ATA {}:{} {} {} ({} {})\n", bus, drive, model, serial, size, unit);
    // }
}

// Return a Vec of info tuples (bus, drive, model, serial, size, unit) for every present drive.
pub fn list() -> Vec<(u8, u8, String, String, u32, String)> {
    let mut result = Vec::new();
    let mut buses = BUSES.lock();

    for bus_id in 0..buses.len() {
        for drive in 0..2 {
            if let Some(identify_buf) = buses[bus_id].identify_drive(drive as u8) {
                // Extract serial number (words 10..20)
                let mut serial = String::new();
                for word in &identify_buf[10..20] {
                    for &b in &word.to_be_bytes() {
                        serial.push(b as char);
                    }
                }
                let serial = serial.trim().to_string();

                // Extract model string (words 27..47)
                let mut model = String::new();
                for word in &identify_buf[27..47] {
                    for &b in &word.to_be_bytes() {
                        model.push(b as char);
                    }
                }
                let model = model.trim().to_string();

                // Extract total sector count from words 60 (low) and 61 (high)
                let sectors = (identify_buf[61] as u32) << 16 | (identify_buf[60] as u32);
                let (size, unit) = disk_size(sectors);

                result.push((bus_id as u8, drive as u8, model, serial, size, unit));
            }
        }
    }

    result
}

// Top-level read: dispatch to the appropriate Bus instance.
pub fn read(bus: u8, drive: u8, block: u32, buf: &mut [u8]) {
    let mut buses = BUSES.lock();
    buses[bus as usize].read(drive, block, buf);
}

// Top-level write: dispatch to the appropriate Bus instance.
pub fn write(bus: u8, drive: u8, block: u32, buf: &[u8]) {
    let mut buses = BUSES.lock();
    buses[bus as usize].write(drive, block, buf);
}