//! HID++ device discovery and inspection for OpenLogi.
//!
//! Wraps the `hidpp` crate over `async-hid` as the transport. The only public
//! entry point is [`inventory::enumerate`].

mod transport;

pub mod inventory;
pub use inventory::{InventoryError, enumerate};
