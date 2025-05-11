// ATA command codes sent to the drive’s command register
enum Command {
    // Read sectors:
    // Instructs the drive to transfer one or more sectors from the disk into its data register.
    Read = 0x20,

    // Write sectors:
    // Instructs the drive to transfer one or more sectors from its data register out to the disk.
    Write = 0x30,

    // Identify drive:
    // Requests the drive to return a 512-byte block of identification data (model, serial, capabilities).
    Identify = 0xEC,
}

// Status register bits for an ATA device, read from the status register.
enum Status {
    // Error (bit 0):
    // Indicates that the last command resulted in an error.  
    // Check the error register for detailed error information.
    Error = 0,

    // Index (bit 1):
    // Historically signaled the drive head passing the index on removable media (floppy disks).  
    // Rarely used on modern hard drives.
    Index = 1,

    // Corrected Data (bit 2):
    // Indicates that a data error was detected and automatically corrected (e.g., via ECC).
    CorrectedData = 2,

    // Data Request (bit 3):
    // The drive is ready to transfer data.  
    // The host should read from or write to the data register now.
    DataRequest = 3,

    // Service (bit 4):
    // Used in ATAPI (packet) mode to indicate the drive needs service from the host.
    Service = 4,

    // Device Fault (bit 5):
    // Indicates a non-recoverable fault in the device (hardware failure).
    DeviceFault = 5,

    // Device Ready (bit 6):
    // The device is powered up and ready to accept commands (but not necessarily ready to transfer data).
    DeviceReady = 6,

    // Busy (bit 7):
    // The device is busy processing the current command.  
    // Until this bit clears, the host must not send new commands.
    Busy,
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
