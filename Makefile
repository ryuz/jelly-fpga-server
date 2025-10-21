

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


# Cross Compile
.PHONY: aarch64-build
aarch64-build:
	cross build --target aarch64-unknown-linux-gnu --release

.PHONY: arm-build
arm-build:
	cross build --target arm-unknown-linux-gnueabihf --release

.PHONY: kria-run
kria-run: aarch64-build
	scp target/aarch64-unknown-linux-gnu/release/jelly-fpga-server $(KRIA_SSH_ADDRESS):/tmp/jelly-fpga-server
	ssh -t $(KRIA_SSH_ADDRESS) "sudo /tmp/jelly-fpga-server --external --verbose 1"

.PHONY: zybo-z7-run
zybo-z7-run: arm-build
	scp target/arm-unknown-linux-gnueabihf/release/jelly-fpga-server $(ZYBO_Z7_SSH_ADDRESS):/tmp/jelly-fpga-server
	ssh -t $(ZYBO_Z7_SSH_ADDRESS) "sudo /tmp/jelly-fpga-server --external --verbose 1"
