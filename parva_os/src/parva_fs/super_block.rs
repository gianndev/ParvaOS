const SUPERBLOCK_ADDRESS: u32 = 0;
const SECRET_SIGNATURE: &[u8; 8] = b"PARVA_FS";

pub struct SuperBlock {
    signature: &'static[u8; 8],
    version: u8,
    block_size: u32,
    pub block_count: u32,
    pub alloc_count: u32,
}

impl SuperBlock {
    pub fn new() -> Option<Self> {
        if let Some(ref dev) = *super::block_device::BLOCK_DEVICE.lock() {
            Some(Self {
                signature: SECRET_SIGNATURE,
                version: super::VERSION,
                block_size: dev.block_size() as u32,
                block_count: dev.block_count() as u32,
                alloc_count: 0,
            })
        } else {
            None
        }
    }

    pub fn read(&self) {
        // TODO
    }

    pub fn write(&self) {
        // TODO
    }
}