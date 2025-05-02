# When type 'make' it will automatically run 'all'
.DEFAULT_GOAL := all

# Final target to run everything
all: clean image run

.PHONY: image run
.EXPORT_ALL_VARIABLES:

bin = target/x86_64-parva_os/debug/bootimage-parva_os.bin
img = ParvaOS.img

$(img):
	qemu-img create $(img) 32M

image: $(img)
	cargo build
	cargo bootimage
	dd conv=notrunc if=$(bin) of=$(img)

opts = -m 32 -cpu max -hda $(img)

run:
	qemu-system-x86_64 $(opts)

clean:
	rm $(img)

test:
	cargo test