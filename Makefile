# Set 'run' as the default target when 'make' is executed without arguments
.DEFAULT_GOAL := run

run:
	cargo run

build:
	cargo build

clean:
	cargo clean

test:
	cargo test