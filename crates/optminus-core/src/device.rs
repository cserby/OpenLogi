//! Serializable device-model types.
//!
//! These mirror the HID++ types from the `hidpp` crate but live here so the
//! CLI and any future GUI can depend on them without dragging in the protocol
//! crate or its async transport.

use serde::Serialize;

/// What a paired peripheral is. Mirrors `hidpp::receiver::bolt::BoltDeviceKind`
/// but is owned by us so consumers don't depend on `hidpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Numpad,
    Presenter,
    Remote,
    Trackball,
    Touchpad,
    Tablet,
    Gamepad,
    Joystick,
    Headset,
    Unknown,
}

/// Coarse battery bucket reported by the device firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BatteryLevel {
    Critical,
    Low,
    Good,
    Full,
    Unknown,
}

/// Charging state. Mirrors `hidpp 0.2`'s `BatteryStatus` plus `Unknown` for
/// values added in future protocol versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryStatus {
    Discharging,
    Charging,
    ChargingSlow,
    Full,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatteryInfo {
    pub percentage: u8,
    pub level: BatteryLevel,
    pub status: BatteryStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReceiverInfo {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub unique_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairedDevice {
    /// Receiver-assigned slot (1..=6 for Bolt).
    pub slot: u8,
    pub codename: Option<String>,
    /// Wireless product ID. `None` for offline / unreachable devices on hidpp 0.2.
    pub wpid: Option<u16>,
    pub kind: DeviceKind,
    pub online: bool,
    pub battery: Option<BatteryInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInventory {
    pub receiver: ReceiverInfo,
    pub paired: Vec<PairedDevice>,
}
