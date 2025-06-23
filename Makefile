# When type 'make' it will automatically run 'all'
.DEFAULT_GOAL := all

# Final target to run everything
all: clean image run

.PHONY: setup clean image run
.EXPORT_ALL_VARIABLES:

bin = target/x86_64-parva_os/release/bootimage-parva_os.bin
img = ParvaOS.img

$(img):
	qemu-img create $(img) 32M

image: $(img)
	touch parva_os/src/lib.rs
	cargo bootimage --release
	dd conv=notrunc if=$(bin) of=$(img)

opts = -m 32 -cpu max -hda $(img)

run:
	qemu-system-x86_64 $(opts)

clean:
	rm $(img)

setup:
	rustup install nightly
	rustup default nightly
	cargo install bootimage