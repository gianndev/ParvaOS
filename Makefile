# Set 'run' as the default target when 'make' is executed without arguments
.DEFAULT_GOAL := run

run:
	cd parvaos && cargo run

build:
	cd parvaos && cargo build

clean:
	cd parvaos && cargo clean