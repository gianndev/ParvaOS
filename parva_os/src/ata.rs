// ATA command codes sent to the drive’s command register

use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

enum Command {
    Read = 0x20, // Read sectors: Instructs the drive to transfer one or more sectors from the disk into its data register.
    Write = 0x30, // Write sectors: Instructs the drive to transfer one or more sectors from its data register out to the disk.
    Identify = 0xEC, // Identify drive: Requests the drive to return a 512-byte block of identification data (model, serial, capabilities).
}

// Status register bits for an ATA device, read from the status register.
enum Status {
    Error = 0,          // Error (bit 0): Indicates that the last command resulted in an error. Check the error register for detailed error information.
    Index = 1,          // Index (bit 1): Historically signaled the drive head passing the index on removable media (floppy disks). Rarely used on modern hard drives.
    CorrectedData = 2,  // Corrected Data (bit 2): Indicates that a data error was detected and automatically corrected (e.g., via ECC).
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
    irq: u8, // Interrupt Request Line (IRQ) used by the device to notify the CPU (e.g., IRQ 14 or 15 for ATA)
    data_register: Port<u16>, // 16-bit data register: used to transfer sector data to/from the drive.
    error_register: PortReadOnly<u8>, // Read-only error register: contains error codes after failed operations.
    features_register: PortWriteOnly<u8>, // Write-only features register: used to send advanced command features to the drive.
    sector_count_register: Port<u8>, // Specifies the number of sectors to transfer (usually 1).
    lba0_register: Port<u8>, // 8-bit LBA (Logical Block Addressing) low byte (bits 0–7 of the sector address).
    lba1_register: Port<u8>, // LBA mid byte (bits 8–15 of the sector address).
    lba2_register: Port<u8>, // LBA high byte (bits 16–23 of the sector address).
    drive_register: Port<u8>, // Drive/head register: used to select the drive (master/slave) and bits 24–27 of LBA.
    status_register: PortReadOnly<u8>, // Read-only status register: used to check device state (e.g., busy, ready, error).
    command_register: PortWriteOnly<u8>, // Write-only command register: send commands like Read, Write, Identify.
    alternate_status_register: PortReadOnly<u8>, // Read-only alternate status register: same as status but does not clear interrupt flags.
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
}

fn init() {
    // TODO
}

fn read() {
    // TODO
}

fn write() {
    // TODO
}
