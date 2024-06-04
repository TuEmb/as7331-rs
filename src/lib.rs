//! this crate provides as7331 driver for embedded devices (currently support esp chips).
//! please run example first to understand.
//!
//! ## Choosing a device and run example
//!
//! Depending on your target device, you need to enable the chip feature
//! for that device or simply run with alias.
//! - with alias:
//!
//!     ```r_<device> : cargo r_esp32, cargo r_esp32c6, ...```
//! - full command:
//!
//!     ```cargo run --release --features=esp32c2 --target=riscv32imc-unknown-none-elf --example uv```
//!
//!

#![no_std]
pub mod as7331;
pub use as7331::As7331;
