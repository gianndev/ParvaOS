// ATA command codes sent to the drive’s command register
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

pub struct Bus {
    // TODO
}

impl Bus {
    // TODO
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
