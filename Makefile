

.PHONY: all
all: build

.PHONY: build
build:
	cargo build --release

.PHONY: run
run: build
	./target/release/jelly-fpga-server

.PHONY: clean
clean:
	cargo clean


# Cross Compile for Kria
.PHONY: kria-build
kria-build:
	cross build --target aarch64-unknown-linux-gnu --release

.PHONY: kria-run
kria-run: kria-build
	scp target/aarch64-unknown-linux-gnu/release/jelly-fpga-server $(KRIA_BOARD_ADDRESS):/tmp/jelly-fpga-server
	ssh -t $(KRIA_BOARD_ADDRESS) "sudo /tmp/jelly-fpga-server --external"
