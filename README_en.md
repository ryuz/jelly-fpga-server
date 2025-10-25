日本語版は[こちら](README.md)です。

# Overview

`jelly-fpga-server` is a gRPC-based server application targeting Zynq/ZynqMP devices. It is primarily designed for use with Zynq UltraScale+ devices (such as Kria KV260).

By running a gRPC server as a service to control PL (Programmable Logic) components that require root privileges, this tool streamlines FPGA development.

It is accessible not only from within the board but also over the network, enabling seamless development when cross-compiling from high-performance PCs running Vivado and other tools.

Note that this is intended for research and development purposes and does not include authentication features, so use within a local network is recommended.

# Environment

Currently tested and verified on the following environments:

- [Kria KV260](https://www.amd.com/en/products/system-on-modules/kria/k26/kv260-vision-starter-kit.html) + [Ubuntu 24.04](https://ubuntu.com/download/amd)
- [ZYBO Z7-20](https://digilent.com/reference/programmable-logic/zybo-z7/) + [Debian12](https://github.com/ikwzm/FPGA-SoC-Debian12)

Primarily intended for FPGA bitstream downloads using [jelly-fpga-loader](https://github.com/ryuz/jelly-fpga-loader) and FPGA operations from various language bindings:

- Rust : [jelly-fpga-client-rs](https://github.com/ryuz/jelly-fpga-client-rs)
- Python : [jelly-fpga-client-py](https://github.com/ryuz/jelly-fpga-client-py)
- Elixir : [jelly-fpga-client-ex](https://github.com/ryuz/jelly-fpga-client-ex)


# Installation

Follow these steps to install the server on the FPGA board's Linux system.

## Prerequisites

Since we use dtc (Device Tree Compiler) and bootgen, install them with the following commands:

```bash
sudo apt update
sudo apt install libssl-dev dtc
```

```bash
git clone https://github.com/Xilinx/bootgen
cd bootgen/
make
sudo cp bootgen /usr/local/bin/
```

## Binary Installation

Execute the following command to download and install the latest binary from the GitHub releases page:

```bash
curl -LsSf https://raw.githubusercontent.com/ryuz/jelly-fpga-server/master/binst.sh | sudo bash
```

## Build and Install from Source

Rust installation is required. Install it with the following commands:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Protocol Buffers compiler (protoc) installation is also required:

```bash
sudo apt update
sudo apt install protobuf-compiler
```

Clone the repository and run the installation script:

```bash
git clone https://github.com/ryuz/jelly-fpga-server.git
cd jelly-fpga-server
sudo ./install.sh
```

## Uninstallation

Uninstall with the following commands:

```bash
git clone https://github.com/ryuz/jelly-fpga-server.git
cd jelly-fpga-server
sudo ./uninstall.sh
```

Or manually uninstall as follows:

```bash
sudo systemctl stop jelly-fpga-server
sudo systemctl disable jelly-fpga-server
sudo rm /usr/local/bin/jelly-fpga-server
sudo rm /etc/systemd/system/jelly-fpga-server.service
sudo rm /etc/default/jelly-fpga-server
```

## Usage

Start the service:

```bash
sudo systemctl start jelly-fpga-server
```

Check service status:

```bash
sudo systemctl status jelly-fpga-server
```

Stop the service:

```bash
sudo systemctl stop jelly-fpga-server
```


## Environment Configuration File

You can specify service startup options by setting environment variables in `/etc/default/jelly-fpga-server`.

```bash
Options:
  -v, --verbose <VERBOSE>  Verbosity level (0-2) [default: 0]
      --external           Allow external connections
  -p, --port <PORT>        Listening port [default: 8051]
      --allow-sudo         Allow execution with sudo privileges
  -h, --help               Show help message
  -V, --version            Show version information
```

By default, --external is enabled in binary installation, allowing external connections. Change as needed.




# Main Features Available via gRPC Remote Execution

## Feature Overview

- Load bitstreams to FPGA
- Apply device tree overlays
- Register/unregister accelerators managed by xmutil or dfx-mgr-client
- Memory-mapped I/O access
- Userspace I/O device access
- Access to [u-dma-buf](https://github.com/ikwzm/udmabuf)


## API Specification

The server provides the following gRPC services:

### FPGA Control
- `Reset`: System reset
- `Load`: Load bitstream
- `Unload`: Unload bitstream

### Firmware Management
- `UploadFirmware`: Upload firmware (streaming)
- `RemoveFirmware`: Remove firmware
- `LoadBitstream`: Load bitstream
- `LoadDtbo`: Load device tree overlay

### Memory Accessors
- `OpenMmap`: Create memory-mapped accessor
- `OpenUio`: Create UIO accessor
- `OpenUdmabuf`: Create UDMABUF accessor
- `Subclone`: Create sub-accessor
- `Close`: Close accessor

### Memory Operations
- `WriteMemU/I`: Write integers to memory
- `ReadMemU/I`: Read integers from memory
- `WriteRegU/I`: Write integers to registers
- `ReadRegU/I`: Read integers from registers
- `WriteMemF32/F64`: Write floating-point to memory
- `ReadMemF32/F64`: Read floating-point from memory
- `MemCopyTo/From`: Copy byte arrays

See `protos/jelly_fpga_control.proto` for detailed API specifications.


## Related Projects

- [jelly-fpgautil-rs](https://github.com/ryuz/jelly-fpgautil-rs) - FPGA control utilities
- [jelly-uidmng-rs](https://github.com/ryuz/jelly-uidmng-rs) - UID management utilities
- [jelly-mem_access](https://crates.io/crates/jelly-mem_access) - Memory access library


## Related Articles

Articles related to this software are available below:

- [jelly-fpga-server explanation article (Zenn)](https://zenn.dev/ryuz88/articles/jelly-fpga-server)
