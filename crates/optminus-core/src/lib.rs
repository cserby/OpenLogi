//! Shared types and configuration for OptMinus.
//!
//! This crate is deliberately I/O-free apart from filesystem reads/writes of
//! the user config file. It must never depend on `hidpp`, `async-hid`, or any
//! platform-specific event/window API — those live in sibling crates.

pub mod config;
pub mod device;
pub mod paths;
