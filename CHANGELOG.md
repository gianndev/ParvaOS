# Changelog

## x.x.x (xxxx-xx-xx)

## 0.0.2 (2025-05-02)
- **GUI**: added a window manager that shows windows on screen
- **Terminal**: for now ony one window is showed and it is the terminal, so everything happens inside this window
- **Makefile**: changed so that the release file is an image file and not just a binary

## 0.0.1 (2025-04-28)
- **x86_64 CPU**: supported x86 64-bit as CPU architecture
- **VGA Text Mode**: added as TUI
- **Serial output**: added to make internal tests
- **CPU Exceptions**: added a way to manage CPU exception without crashing the entire system
- **Paging**: implemented virtual memory management by introducing paging, enabling memory isolation and efficient use of physical memory.
- **Heap allocation**: introduced a dynamic memory allocator to manage memory allocation and deallocation at runtime, enabling more flexible and efficient use of system resources.
- **Shell**: added a basic shell with a cursor, backspace working an `hello` as example command
- Added `help` and `info` commands

## 0.0.0 (2025-04-24)
- Started ParvaOS project