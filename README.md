# The-Gift

This is an embendded project of `NB-IoT` + `Risc-V MCU` + `E-Paper Display`, with hardware designed at <https://oshwhub.com/tnze/the-gift>.

## Build

### Install Requirements

1. Install Rust <https://www.rust-lang.org/learn/get-started>
2. Install `nightly` toolchain  
`rustup toolchain install nightly`
3. Install `riscv32imac-unknown-none-elf` target  
`rustup target add riscv32imac-unknown-none-elf`
4. Extract `tools.7z` for other toolchains you will need.  
You may also need to add theirs `bin` dir into your `PATH`.

### Build the Firmware

```ps
cargo b -r
riscv-nuclei-elf-objcopy -O binary target\riscv32imac-unknown-none-elf\release\the-gift firmware.bin
```

### Run firmware.bin in The Device

```ps
dfu-util -a 0 -s 0x08000000:leave -D firmware.bin
```
