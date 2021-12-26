# Rust ESP32-S2 Starter

A starter for ESP32-S2 projects, including tooling to build the "Cargo-first" way with [`embuild`](https://github.com/ivmarkov/embuild).

This is the result of some work over the past few weeks getting into both Rust development, and working with the ESP32-S2 MCU. The end-goal is to build a few WiFi-connected air quality sensors to place around my house. That project aims to use 3,000mAh Li-Ion 18650 batteries and (hopefully!) year-plus operation, so ultra-low power usage is critical.

This project includes a buildable example that can be flashed to your ESP32-S2, including some boilerplate code to interact with the ULP coprocessor.

*Please note that I am new to Cargo Workspaces, Rust, and the ESP32. Please open an issue if you notice anything wrong!*

<br />

## Todo

- [x] Build both main and ULP binaries from one workspace
- [x] Build the main binary using embuild's `native` features (e.g. no PlatformIO)
- [x] Allow flashing from the WSL2 environment
- [ ] Example ULP code to sleep for a specified interval, then wake-up the main processor
- [ ] Example code to comminicate between main and ULP processors
- [ ] Allow building via native `cargo build`

<br />

## Getting Started
 * Install rustup from https://rustup.rs/
 * [Install the custom Rust ESP32 toolchain and LLVM Clang forks](https://github.com/esp-rs/rust-build#rust-build)
   - You probably want to do the following in `~/.espressif-rust` or `/opt/rustup-esp` or similar
   - `curl -sSL -o install-rust-toolchain.sh https://github.com/esp-rs/rust-build/releases/download/v1.57.0.2/install-rust-toolchain.sh`
   - `sudo chmod u+x install-rust-toolchain.sh`
   - `./install-rust-toolchain.sh --export-file ./rustup-export.sh`
   - `source ./rustup-export.sh` (this needs to be done for each bash session where you're building the ESP32-S2 project)
 * Download the starter
   - `git clone https://github.com/ryanc-me/esp32s2-starter`

### For WSL2 users

Some extra setup is required to allow flashing the chip from the WSL2 environment.
 * From the **Windows** environment
   - Install Rust: https://www.rust-lang.org/tools/install
   - `cargo install espflash`
   - `cargo install espmonitor`

<br />

## Building

Because of some Cargo limitations (or lack of understanding on my part) the native `cargo build` does not work from the workspace root. From what I can gather, Rust is trying to use the same toolchain for both main and ULP crates.

As a workaround, you can build each crate separately, from within the crate directory:
 * `cd ulp && cargo build`
 * `cd main && cargo build --features native`

Alternatively, there is a `build.sh` script which does the above for you.

<br />

## Flashing

### For Linux users

Follow the standard flashing procedure, e.g.:
 * Ensure tools are installed
   - `cargo install espflash`
   - `cargo install espmonitor`
 * `espflash --monitor /dev/ttyUSB1 target/xtensa-esp32s2-espidf/debug/main`
 * `espmonitor /dev/ttyUSB1`

### For Windows/WSL2 users

 * Make sure both `espflash` and `espmonitor` are installed in the **Windows** environment.
   - Install Rust
   - `cargo install espflash`
   - `cargo install espmonitor`
 * Then, from your **WSL2 Linux** environment:
   - `powershell.exe -Command "espflash --monitor COM1 target/xtensa-esp32s2-espidf/debug/main`
   - `powershell.exe -Command "espmonitor COM1"`
   - Note that `powershell.exe` will only be available in your Linux env if you open your WSL2 target e.g. via Windows Terminal, or VSCode's `Remote WSL` extension (the latter is a very good option). Some info [here](https://superuser.com/questions/1538580/using-ssh-powershell-to-run-windows-programs-on-a-win10-machine-with-wsl) if, for some reason, you need to do this via SSH.
 * Or, alternatively, the `flash.sh` script runs the flash+monitor command for you.

<br />

## Credits
 * [Ivan Markov](https://github.com/ivmarkov) for [`rust-esp32-std-demo`](https://github.com/ivmarkov/rust-esp32-std-demo) (which the `main` crate is based on)
 * [Ivan Markov](https://github.com/ivmarkov) for [`rust-esp32-ulp-blink`](https://github.com/ivmarkov/rust-esp32-std-demo) (which the `ulp` crate is based on)

<br />

## License

 * [APACHE](./LICENSE-APACHE)
 * [MIT](./LICENSE-MIT)
