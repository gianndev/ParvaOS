## **ParvaFS**

**ParvaFS is the official file system of ParvaOS.** This guide explains how ParvaFS organizes and manages data on disk, covering its design decisions, data structures, and core operations.

---

### **Theoretical Basis of File Systems**

A file system organizes storage into structured blocks. Key concepts:

* **Superblock**: Stores file system metadata, such as a magic signature, version, block size, and pointers to other regions.
* **Bitmap Blocks**: Maintain a bit-level map of which data blocks are free or allocated, enabling quick allocation and deallocation.
* **Data Blocks**: Fixed-size units (512 bytes each) that hold file contents or directory entries.
* **Directory Entries**: Metadata records for files/directories, including type (file or directory), size, starting block address, and name.
* **Directory Structure**: A linked-chain of blocks representing a directory’s entries, enabling variable-sized directories.

---

## **ParvaFS Layout on Disk**

ParvaFS divides the disk into several logical regions:

1. **Reserved Region**: First block (before superblock), reserved for boot code or other uses.
2. **Superblock (512 bytes)**

   * Contains the magic signature `"PARVA FS"` followed by padding zeros.
   * Written at a fixed address: `SUPERBLOCK_ADDR = (1 << 20) / 512`.
3. **Bitmap Region**

   * Starts at `BITMAP_ADDR_OFFSET = SUPERBLOCK_ADDR + 2`.
   * Consists of `MAX_BLOCKS/8` blocks, each with a 4-byte header and 508 bytes of bitmap data.
   * Each bit represents one data block: 0 = free, 1 = allocated.
4. **Data Region**

   * Begins at `DATA_ADDR_OFFSET = BITMAP_ADDR_OFFSET + (MAX_BLOCKS/8)`.
   * Used to store directory and file data in 512‑byte blocks.

```text
| Reserved | Superblock | Bitmap Blocks | Data Blocks |
```

---

## **Core Data Structures**

### **Block**

* Fixed 512‑byte buffer: first 4 bytes store a `next` block pointer (big‑endian u32), remaining 508 bytes are data.
* Methods:

  * `read(addr)`: load block from disk.
  * `write()`: flush buffer to disk.
  * `alloc()`: find a free data block via `BlockBitmap` and initialize it.
  * `next()`: return next chained block if pointer ≠ 0.
  * `set_next(addr)`: update `next` pointer.

### **BlockBitmap**

* Scans bitmap blocks to find free blocks.
* Methods:

  * `is_free(addr)`: check bit in bitmap for data block at `addr`.
  * `alloc(addr)`: mark bit as 1 (used).
  * `free(addr)`: mark bit as 0 (free).
  * `next_free_addr()`: linear scan to return the next free block address.

### **Directory Entry (`DirEntry`)**

* Represents a file or directory within a `Dir`.
* Fields:

  * `kind: FileType` (Dir or File)
  * `addr: u32` starting block of file contents or subdirectory.
  * `size: u32` number of bytes (for files).
  * `name: String`
* Methods:

  * `to_file()`, `to_dir()`: convert into `File` or `Dir` object.

### **Directory (`Dir`)**

* Represents a directory: stores its starting block address.
* A directory’s blocks form a linked list, each block containing back‑to‑back entries of variable length:

  * 1 byte type, 4 bytes address, 4 bytes size, 1 byte name length, N bytes name.
* Methods:

  * `root()`: return root directory at `DATA_ADDR_OFFSET`.
  * `open(path)`: resolve each component, walking chained blocks.
  * `create_dir(name)`, `create_file(name)`: append new entry and allocate block for its data.
  * `delete_entry(name)`: zero out entry pointer and free all data blocks.
  * `update_entry_size(name, size)`: update size bytes in entry header.
  * `read()`: return `ReadDir` iterator.

### **Read Directory Iterator (`ReadDir`)**

* Iterates entries in a `Dir` by scanning the data region:

  1. Parse the entry header.
  2. Read the name.
  3. Skip entries with addr = 0 (deleted).
  4. Advance to next block when needed.

### **File**

* In‑memory handle for reading/writing a file’s data blocks and metadata.
* Fields:

  * `name`, `addr` (start), `size`, `dir` (parent).
* Read/Write:

  * `read(buf)`: sequentially read through chained blocks.
  * `write(buf)`: overwrite existing blocks, chain new ones, update `Dir` entry size.
  * `delete(path)`: wrapper over `Dir::delete_entry`.

### **BlockDevice**

* Thin wrapper around ATA driver `ata::read/write(bus, dsk, block, buf)`.
* Used via a global `Mutex<Option<BlockDevice>>` guard.

---

## **File System Operations**

### **Mounting**

* `init()`: probe ATA buses/disks for the ParvaFS magic in superblock; if found, call `mount(bus,dsk)`.
* `mount(bus, dsk)`: set the global `BLOCK_DEVICE` to enable all FS calls.

### **Formatting**

* `format(bus, dsk)`: write the magic to superblock, mount the device, and allocate the root directory block.

### **Path Handling**

* `dirname(path)`, `filename(path)`: split full paths at the last `/`.
* `realpath(path)`: convert relative paths to absolute using process’s current directory.

### **File/Directory Lifecycle**

1. **Create**

   * Walk to parent `Dir` via `Dir::open`.
   * Call `create_file` or `create_dir` to append a `DirEntry`, allocate a data block.
2. **Open**

   * Resolve full path to `Dir`; find entry in last component; convert to `File` or `Dir`.
3. **Read**

   * For files: sequentially load each block’s data.
   * For directories: iterate entries via `ReadDir`.
4. **Write**

   * Overwrite existing data blocks, allocate new ones if buffer larger than current chain, update `size` in directory header.
5. **Delete**

   * Zero-out entry pointer, then free all data blocks in chain.

---

### **Performance and Constraints**

* **Fixed block size** (512 bytes) simplifies on‑disk layout but can lead to internal fragmentation.
* **Linked blocks** allow files/directories to grow arbitrarily but incur pointer overhead and slower seeks.
* **Linear bitmap scan** in `next_free_addr()` can be slow for large disks; could be improved with hierarchical bitmaps.

---

### **Acknowledgements**

Some aspects of ParvaFS were inspired by the [moros](https://github.com/vinc/moros) project.