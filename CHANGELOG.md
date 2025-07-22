# Changelog

## 0.0.5 (xxxx-xx-xx)
- **Removed `hello` command**: the terminal window isn't very high so every space counts, so I removed this test command, even though it's a historical command.
- **Added `neofetch` command**: a toy command to just flex that you use ParvaOS btw!

## 0.0.4 (2025-05-18)
- **Processes**: added the ability to create multiple processes inside the OS
- **Added PIT support**: PIT is a Programmable Interval Timer, like a clock simulation
- **Added ATA driver**: this will be useful to save data permanently on a disk image
- **Implemented ParvaFS**: in ParvaOS I have implemented a file system created specifically for this project called ParvaFS
- **Added `install` command**: useful to format the .img file with the ParvaFS file system
- **Added `crfile` and `list` commands**: useful to create a file with a give name and have a list of all existing files
- **Added `read` command**: needed to read the text content of a file
- **Added `edit` command**: needed to edit (more precisely overwrite) the content of a file

## 0.0.3 (2025-05-08)
- **Removed flickering**: now the GUI updates only the changed part of the screen and not the whole screen every time
- **Window movimng**: added the ability to move windows with WASD keys in move_mode
- **Added `clear` command**
- **Added `shutdown` command**
- **Added `reboot` command**
- **Added Fullscreen**: now when the user is in move mode, pressing SPACE he can toggle fullscreen, and pressing SPACE another time he can make the window small again
- **Text logic improved**: improved the logic with which commands and text are displayed in the various windows
- **Completely changed how screen is refreshed**: now only the cells updated are refreshed, to finally solve screen flickering

## 0.0.2 (2025-05-02)
- **GUI**: added a window manager that shows windows on screen
- **Terminal**: for now ony one window is showed and it is the terminal, so everything happens inside this window
- **Makefile**: changed so that the release file is an image file and not just a binary

## 0.0.1 (2025-04-28)
- **x86_64 CPU**: supported x86 64-bit as CPU architecture
- **VGA Text Mode**: added as TUI
- **Serial output**: added to make internal tests
- **CPU Exceptions**: added a way to manage CPU exception without crashing the entire system
- **Paging**: implemented virtual memory management by introducing paging, enabling memory isolation and efficient use of physical memory
- **Heap allocation**: introduced a dynamic memory allocator to manage memory allocation and deallocation at runtime, enabling more flexible and efficient use of system resources
- **Shell**: added a basic shell with a cursor, backspace working an `hello` as example command
- Added `help` and `info` commands

## 0.0.0 (2025-04-24)
- Started ParvaOS project