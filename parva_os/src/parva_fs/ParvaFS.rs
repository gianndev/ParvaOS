// ParvaFS: A simple file system implementation for ParvaOS using ATA block device

use alloc::{borrow::ToOwned, format};
use alloc::string::String;
use alloc::vec::Vec;
use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{ata, println, process};

// Global optional block device handle protected by a Mutex
lazy_static! {
    pub static ref BLOCK_DEVICE: Mutex<Option<BlockDevice>> = Mutex::new(None);
}

// Magic signature for identifying a ParvaFS-formatted disk
const MAGIC: &'static str = "PARVA FS";

// FileType enumeration: distinguishes directories from regular files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Dir = 0,
    File = 1,
}

// Extract the directory component of a pathname
pub fn dirname(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(0) => 1,       // if path starts with '/', root dir
        Some(i) => i,        // otherwise split at last '/'
        None => n,           // no slash => empty dirname (current dir)
    };
    &pathname[0..i]
}

// Extract the filename component of a pathname
pub fn filename(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(i) => i + 1,    // start after last '/'
        None => 0,            // no slash => whole name
    };
    &pathname[i..n]
}

// Convert a relative pathname to an absolute one using current process directory
pub fn realpath(pathname: &str) -> String {
    if pathname.starts_with("/") {
        pathname.into()    // already absolute
    } else {
        let dirname = process::dir();
        let sep = if dirname.ends_with("/") { "" } else { "/" };
        format!("{}{}{}", dirname, sep, pathname)
    }
}

// Representation of an open file: name, starting block address, size, and parent directory
#[derive(Clone)]
pub struct File {
    name: String,
    addr: u32,
    size: u32,
    dir: Dir, // parent directory
}

impl File {
    // Create a new file at the given pathname
    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_file(filename) {
                return Some(dir_entry.to_file());
            }
        }
        None
    }

    // Open an existing file if it exists and is a regular file
    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.find(filename) {
                if dir_entry.is_file() {
                    return Some(dir_entry.to_file());
                }
            }
        }
        None
    }

    // Return file size in bytes
    pub fn size(&self) -> usize {
        self.size as usize
    }

    // Read file data into provided buffer, returning number of bytes read
    pub fn read(&self, buf: &mut [u8]) -> usize {
        let buf_len = buf.len();
        let mut addr = self.addr;
        let mut i = 0;
        loop {
            let block = Block::read(addr);
            let data = block.data();
            let data_len = data.len();
            for j in 0..data_len {
                // stop if buffer full or reached file size
                if i == buf_len || i == self.size() {
                    return i;
                }
                buf[i] = data[j];
                i += 1;
            }
            match block.next() {
                Some(next_block) => addr = next_block.addr(),
                None => return i,  // no more blocks
            }
        }
    }

    // Read entire file into a UTF-8 string
    pub fn read_to_string(&self) -> String {
        let mut buf: Vec<u8> = Vec::with_capacity(self.size());
        buf.resize(self.size(), 0);
        let bytes = self.read(&mut buf);
        buf.resize(bytes, 0);
        String::from_utf8(buf).unwrap()
    }

    // Write buffer to file, allocating or freeing blocks as needed
    pub fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
        let buf_len = buf.len();
        let mut addr = self.addr;
        let mut i = 0;
        while i < buf_len {
            let mut block = Block::new(addr);
            let data = block.data_mut();
            let data_len = data.len();
            // fill block with data
            for j in 0..data_len {
                if i == buf_len {
                    break;
                }
                data[j] = buf[i];
                i += 1;
            }

            addr = match block.next() {
                Some(next_block) => {
                    if i < buf_len {
                        next_block.addr() // continue writing
                    } else {
                        0 // no next block when done
                    }
                }
                None => {
                    if i < buf_len {
                        // need a new block
                        match Block::alloc() {
                            Some(next_block) => next_block.addr(),
                            None => return Err(()),
                        }
                    } else {
                        0
                    }
                }
            };

            // update block chaining and write to disk
            block.set_next(addr);
            block.write();
        }
        // update file metadata
        self.size = i as u32;
        self.dir.update_entry_size(&self.name, self.size);
        Ok(())
    }

    // Return starting block address of file
    pub fn addr(&self) -> u32 {
        self.addr
    }

    // Delete a file by pathname
    pub fn delete(pathname: &str) -> Result<(), ()> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            dir.delete_entry(filename)
        } else {
            Err(())
        }
    }
}

// 512-byte block: 4-byte next pointer + 508-byte data
#[derive(Clone)]
pub struct Block {
    addr: u32,
    buf: [u8; 512],
}

impl Block {
    // Create an empty block buffer at given address
    pub fn new(addr: u32) -> Self {
        let buf = [0; 512];
        Self { addr, buf }
    }

    // Read block data from device into buffer
    pub fn read(addr: u32) -> Self {
        let mut buf = [0; 512];
        if let Some(ref block_device) = *BLOCK_DEVICE.lock() {
            block_device.read(addr, &mut buf);
        }
        Self { addr, buf }
    }

    // Allocate a free block using the bitmap
    pub fn alloc() -> Option<Self> {
        match BlockBitmap::next_free_addr() {
            None => None,
            Some(addr) => {
                BlockBitmap::alloc(addr);
                let mut block = Block::read(addr);
                // zero-initialize
                for i in 0..512 {
                    block.buf[i] = 0;
                }
                block.write();
                Some(block)
            }
        }
    }

    // Write block buffer to device
    pub fn write(&self) {
        if let Some(ref block_device) = *BLOCK_DEVICE.lock() {
            block_device.write(self.addr, &self.buf);
        }
    }

    // Return block address
    pub fn addr(&self) -> u32 { self.addr }

    // Return immutable view of data region
    pub fn data(&self) -> &[u8] {
        &self.buf[4..512]
    }

    // Return mutable view of data region
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.buf[4..512]
    }

    // Read next chained block if present
    pub fn next(&self) -> Option<Self> {
        let addr = (self.buf[0] as u32) << 24
                 | (self.buf[1] as u32) << 16
                 | (self.buf[2] as u32) << 8
                 | (self.buf[3] as u32);
        if addr == 0 {
            None
        } else {
            Some(Self::read(addr))
        }
    }

    // Set next block pointer
    pub fn set_next(&mut self, addr: u32) {
        self.buf[0] = addr.get_bits(24..32) as u8;
        self.buf[1] = addr.get_bits(16..24) as u8;
        self.buf[2] = addr.get_bits(8..16) as u8;
        self.buf[3] = addr.get_bits(0..8) as u8;
    }
}

// Bitmap parameters for tracking free blocks
const BITMAP_SIZE: u32 = 512 - 4; // data bytes in bitmap block
const MAX_BLOCKS: u32 = 2 * 2048;
const DISK_OFFSET: u32 = (1 << 20) / 512;
const SUPERBLOCK_ADDR: u32 = DISK_OFFSET;
const BITMAP_ADDR_OFFSET: u32 = DISK_OFFSET + 2;
const DATA_ADDR_OFFSET: u32 = BITMAP_ADDR_OFFSET + MAX_BLOCKS / 8;

// BlockBitmap: manage allocation status of data blocks via bitmap stored on disk
pub struct BlockBitmap {}

impl BlockBitmap {
    // Compute bitmap block index for a data block address
    fn block_index(data_addr: u32) -> u32 {
        let i = data_addr - DATA_ADDR_OFFSET;
        BITMAP_ADDR_OFFSET + (i / BITMAP_SIZE / 8)
    }

    // Compute byte offset inside bitmap block
    fn buffer_index(data_addr: u32) -> usize {
        let i = data_addr - DATA_ADDR_OFFSET;
        (i % BITMAP_SIZE) as usize
    }

    // Check if a block is free
    pub fn is_free(addr: u32) -> bool {
        let block = Block::read(BlockBitmap::block_index(addr));
        let bitmap = block.data();
        let i = BlockBitmap::buffer_index(addr);
        bitmap[i / 8].get_bit(i % 8)
    }

    // Mark a block as allocated
    pub fn alloc(addr: u32) {
        let mut block = Block::read(BlockBitmap::block_index(addr));
        let bitmap = block.data_mut();
        let i = BlockBitmap::buffer_index(addr);
        bitmap[i / 8].set_bit(i % 8, true);
        block.write();
    }

    // Mark a block as free
    pub fn free(addr: u32) {
        let mut block = Block::read(BlockBitmap::block_index(addr));
        let bitmap = block.data_mut();
        let i = BlockBitmap::buffer_index(addr);
        bitmap[i / 8].set_bit(i % 8, false);
        block.write();
    }

    // Find next free data block address by scanning bitmap
    pub fn next_free_addr() -> Option<u32> {
        let n = MAX_BLOCKS / BITMAP_SIZE / 8;
        for i in 0..n {
            let block = Block::read(BITMAP_ADDR_OFFSET + i);
            let bitmap = block.data();
            for j in 0..BITMAP_SIZE {
                for k in 0..8 {
                    if !bitmap[j as usize].get_bit(k) {
                        let addr = DATA_ADDR_OFFSET + i * 512 * 8 + j * 8 + k as u32;
                        return Some(addr);
                    }
                }
            }
        }
        None
    }
}

// Directory entry metadata: parent Dir, type, address, size, and name
#[derive(Clone)]
pub struct DirEntry {
    dir: Dir,
    kind: FileType,
    addr: u32,
    size: u32,
    name: String,
}

impl DirEntry {
    // Construct a new DirEntry
    pub fn new(dir: Dir, kind: FileType, addr: u32, size: u32, name: &str) -> Self {
        let name = String::from(name.to_owned());
        Self { dir, kind, addr, size, name }
    }
    // Check if entry is directory
    pub fn is_dir(&self) -> bool { self.kind == FileType::Dir }
    // Check if entry is file
    pub fn is_file(&self) -> bool { self.kind == FileType::File }
    pub fn size(&self) -> u32 { self.size }
    pub fn name(&self) -> String { self.name.clone() }
    // Convert entry to Dir object
    pub fn to_dir(&self) -> Dir {
        assert!(self.kind == FileType::Dir);
        Dir { addr: self.addr }
    }
    // Convert entry to File object
    pub fn to_file(&self) -> File {
        assert!(self.kind == FileType::File);
        File { name: self.name.clone(), addr: self.addr, size: self.size, dir: self.dir }
    }
    // Compute byte length of entry on disk
    pub fn len(&self) -> usize {
        1 + 4 + 4 + 1 + self.name.len()
    }
}// Directory abstraction managing entries by chaining blocks together
#[derive(Clone, Copy)]
pub struct Dir {
    addr: u32, // Starting block address of this directory
}

impl Dir {
    // Return the root directory, which lives at a fixed offset in the data region
    pub fn root() -> Self {
        Self { addr: DATA_ADDR_OFFSET }
    }

    // Create a new directory at the given (possibly relative) path
    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);               // Make absolute
        let dirname = dirname(&pathname);                // Parent path
        let filename = filename(&pathname);              // New dir name
        // If parent exists, create the new subdirectory entry
        if let Some(dir) = Dir::open(dirname) {
            if let Some(entry) = dir.create_dir(filename) {
                return Some(entry.to_dir());
            }
        }
        None
    }

    // Open an existing directory by walking each component from root
    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let mut dir = Dir::root();                       // Start at root

        if !is_mounted() {                               // FS must be mounted
            return None;
        }

        if pathname == "/" {                             // Special-case root
            return Some(dir);
        }

        // Walk each path component
        for name in pathname.trim_start_matches('/').split('/') {
            match dir.find(name) {
                Some(de) if de.is_dir() => {
                    dir = de.to_dir();                   // Descend into subdir
                }
                _ => return None,                        // Missing or not a dir
            }
        }
        Some(dir)
    }

    // Get this directory's block address
    pub fn addr(&self) -> u32 {
        self.addr
    }

    // Find an entry by name in this directory, returning its metadata
    pub fn find(&self, name: &str) -> Option<DirEntry> {
        for entry in self.read() {
            if entry.name == name {
                return Some(entry);
            }
        }
        None
    }

    // Create a new file entry in this directory
    pub fn create_file(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::File, name)
    }

    // Create a new subdirectory entry in this directory
    pub fn create_dir(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Dir, name)
    }

    // Core routine to append a DirEntry (file or dir) to this directory
    fn create_entry(&self, kind: FileType, name: &str) -> Option<DirEntry> {
        // Skip if name already exists
        if self.find(name).is_some() {
            return None;
        }

        // Iterate to the last block of the directory
        let mut rd = self.read();
        while rd.next().is_some() {}

        // If there's not enough space for the new entry header+name, allocate a new block
        if rd.block.data().len() - rd.data_offset < name.len() + 10 {
            let nb = Block::alloc().unwrap();
            rd.block.set_next(nb.addr);
            rd.block.write();
            rd.block = nb;
            rd.data_offset = 0;
        }

        // Allocate a fresh block to hold the file/dir's data
        let entry_block = Block::alloc().unwrap();
        let entry_addr  = entry_block.addr();
        let entry_size  = 0;                // newly created entries start with size 0
        let entry_name  = name.as_bytes();
        let n           = entry_name.len();
        let i           = rd.data_offset;
        let data        = rd.block.data_mut();

        // Write entry header:
        data[i + 0] = kind as u8;                         // FileType
        // 4-byte big-endian addr of first block
        data[i + 1] = entry_addr.get_bits(24..32) as u8;
        data[i + 2] = entry_addr.get_bits(16..24) as u8;
        data[i + 3] = entry_addr.get_bits(8..16) as u8;
        data[i + 4] = entry_addr.get_bits(0..8) as u8;
        // 4-byte initial size (0)
        data[i + 5] = entry_size.get_bits(24..32) as u8;
        data[i + 6] = entry_size.get_bits(16..24) as u8;
        data[i + 7] = entry_size.get_bits(8..16) as u8;
        data[i + 8] = entry_size.get_bits(0..8) as u8;
        // Name length
        data[i + 9] = n as u8;
        // Name bytes
        for j in 0..n {
            data[i + 10 + j] = entry_name[j];
        }
        rd.block.write();

        // Return a DirEntry wrapper for the new file/dir
        Some(DirEntry::new(self.clone(), kind, entry_addr, entry_size, name))
    }

    // Remove (delete) an entry by name: zero its addr and free all its blocks
    pub fn delete_entry(&mut self, name: &str) -> Result<(), ()> {
        let mut rd = self.read();
        for entry in &mut rd {
            if entry.name == name {
                // Zero-out the stored block address to mark deletion
                let data = rd.block.data_mut();
                let i = rd.data_offset - entry.len();
                data[i + 1] = 0;
                data[i + 2] = 0;
                data[i + 3] = 0;
                data[i + 4] = 0;
                rd.block.write();

                // Walk and free each chained block belonging to this entry
                let mut blk = Block::read(entry.addr);
                loop {
                    BlockBitmap::free(blk.addr);
                    match blk.next() {
                        Some(nb) => blk = nb,
                        None => break,
                    }
                }
                return Ok(());
            }
        }
        Err(())
    }

    // Update the size field in the directory entry header after a write
    fn update_entry_size(&mut self, name: &str, size: u32) {
        let mut rd = self.read();
        for entry in &mut rd {
            if entry.name == name {
                let data = rd.block.data_mut();
                let i = rd.data_offset - entry.len();
                data[i + 5] = size.get_bits(24..32) as u8;
                data[i + 6] = size.get_bits(16..24) as u8;
                data[i + 7] = size.get_bits(8..16) as u8;
                data[i + 8] = size.get_bits(0..8) as u8;
                rd.block.write();
                break;
            }
        }
    }

    // Begin iterating over entries in this directory
    pub fn read(&self) -> ReadDir {
        ReadDir {
            dir: self.clone(),
            block: Block::read(self.addr),
            data_offset: 0,
        }
    }

    // Convenience: delete by full pathname
    pub fn delete(pathname: &str) -> Result<(), ()> {
        let pathname = realpath(pathname);
        let dirname  = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            dir.delete_entry(filename)
        } else {
            Err(())
        }
    }
}

// Iterator over directory entries
pub struct ReadDir {
    dir: Dir,             // Directory being iterated
    block: Block,         // Current block buffer
    data_offset: usize,   // Offset within block.data()
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        loop {
            let data = self.block.data();
            let mut i = self.data_offset;

            // Scan for next valid entry in this block
            loop {
                if i >= data.len() - 10 {
                    break; // Not enough space for another entry header
                }

                // Parse entry header
                let kind = match data[i + 0] {
                    0 => FileType::Dir,
                    1 => FileType::File,
                    _ => break,
                };
                let addr = (data[i + 1] as u32) << 24
                         | (data[i + 2] as u32) << 16
                         | (data[i + 3] as u32) << 8
                         | (data[i + 4] as u32);
                let size = (data[i + 5] as u32) << 24
                         | (data[i + 6] as u32) << 16
                         | (data[i + 7] as u32) << 8
                         | (data[i + 8] as u32);
                i += 9;

                // Read name length
                let mut n = data[i];
                if n == 0 || n as usize > data.len() - i {
                    break;
                }
                i += 1;

                // Read the name characters
                let mut name = String::new();
                while n > 0 {
                    name.push(data[i] as char);
                    i += 1;
                    n -= 1;
                }

                self.data_offset = i;

                // Skip entries marked deleted (addr == 0)
                if addr == 0 {
                    continue;
                }

                // Return the DirEntry
                return Some(DirEntry::new(self.dir, kind, addr, size, &name));
            }

            // Move to next block in chain
            if let Some(nb) = self.block.next() {
                self.block = nb;
                self.data_offset = 0;
            } else {
                break;
            }
        }
        None
    }
}

// Low-level block device wrapper over ATA bus/disk
pub struct BlockDevice {
    bus: u8,
    dsk: u8,
}

impl BlockDevice {
    pub fn new(bus: u8, dsk: u8) -> Self {
        Self { bus, dsk }
    }

    // Read a 512-byte sector into buf
    pub fn read(&self, block: u32, mut buf: &mut [u8]) {
        ata::read(self.bus, self.dsk, block, &mut buf);
    }

    // Write a 512-byte sector from buf
    pub fn write(&self, block: u32, buf: &[u8]) {
        ata::write(self.bus, self.dsk, block, &buf);
    }
}

// Check whether a filesystem has been mounted (block device set)
pub fn is_mounted() -> bool {
    BLOCK_DEVICE.lock().is_some()
}

// Mount a filesystem by setting the global block device handle
pub fn mount(bus: u8, dsk: u8) {
    let bd = BlockDevice::new(bus, dsk);
    *BLOCK_DEVICE.lock() = Some(bd);
}

// Format a disk: write superblock, mount it, allocate root directory block
pub fn format(bus: u8, dsk: u8) {
    // Write MAGIC string to superblock
    let mut buf = MAGIC.as_bytes().to_vec();
    buf.resize(512, 0);
    let block_device = BlockDevice::new(bus, dsk);
    block_device.write(SUPERBLOCK_ADDR, &buf);

    mount(bus, dsk);

    // Mark root dir block as allocated
    let root = Dir::root();
    BlockBitmap::alloc(root.addr());
}

// On OS init: probe each ATA device for the MAGIC superblock and auto-mount it
pub fn init() {
    for bus in 0..2 {
        for dsk in 0..2 {
            let mut buf = [0u8; 512];
            ata::read(bus, dsk, SUPERBLOCK_ADDR, &mut buf);
            if let Ok(header) = String::from_utf8(buf[0..8].to_vec()) {
                if header == MAGIC {
                    println!("ParvaFS Superblock found in ATA {}:{}\n", bus, dsk);
                    mount(bus, dsk);
                }
            }
        }
    }
}