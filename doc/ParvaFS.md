## **ParvaFS**

**ParvaFS is the official file system of ParvaOS**. Here is a guide to understand how this file system works, since it is not an implementation of an existing file system (such as FAT32 or Ext2) ad rather it is something new.

---

### **Theoretical Basis of File Systems**
A file system organizes storage into structured blocks. Key concepts:

- **Superblock**: Metadata about the file system (e.g., block size, total blocks, free blocks).
- **Bitmap Blocks**: Track allocated/free blocks (1 bit per block).
- **Data Blocks**: Store file/directory contents.
- **Inodes/Directory Entries**: Metadata about files (name, size, permissions, block pointers).
- **Directory Structure**: Hierarchical organization of files/directories.