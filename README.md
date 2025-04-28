# ParvaOS

**ParvaOS** is an operating system written from scratch in Rust by Francesco Giannice. It is capable of running on all 64-bit x86 architecture computers with BIOS and has been found to run on QEMU as a virtual machine emulator.

## Features

- x86 CPU support (64 bit)
- VGA Text Mode
- Serial output
- CPU exceptions management
- Paging
- Heap allocation
- Basic shell

## Planned to be implemented
- [ ] Some games
- [ ] More commands in the shell
- [ ] A file system
- [ ] A basic GUI

## How to compile ParvaOS' code?

If you want to compile the whole project on your local machine, follow these instructions:

1. **Install Rust:**

   Rust is required to compile ParvaOS. You can download it from [rust-lang.org](https://www.rust-lang.org/).

2. **Clone the repo:**

    ```
    git clone https://github.com/gianndev/parvaos.git
    cd parvaos
    ```

3. **Build ParvaOS:**

    ```
    make build
    ```

4. **Run ParvaOS:**

    ```
    make run
    ```

## Acknowledgments:
* A special thanks to Phil-Opp's [blog](https://os.phil-opp.com/) 

## License

This project is licensed under the terms of the GNU General Public License v3.0 only (GPL-3.0-only).  
See the [LICENSE](./LICENSE) file for details.