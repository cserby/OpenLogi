//! Enumerate connected HID++ receivers and their paired devices.

use std::{sync::Arc, time::Duration};

use async_hid::HidBackend;
use futures_lite::StreamExt;
use hidpp::{
    channel::HidppChannel,
    device::Device,
    feature::unified_battery::v0::{
        BatteryLevel as HidppBatteryLevel, BatteryStatus as HidppBatteryStatus,
        UnifiedBatteryFeatureV0,
    },
    nibble::U4,
    receiver::{
        self, Receiver,
        bolt::{BoltDeviceConnection, BoltDeviceKind, BoltEvent, BoltReceiver},
    },
};
use optminus_core::device::{
    BatteryInfo, BatteryLevel, BatteryStatus, DeviceInventory, DeviceKind, PairedDevice,
    ReceiverInfo,
};
use thiserror::Error;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::transport::AsyncHidChannel;

/// Logitech HID vendor ID.
const LOGITECH_VID: u16 = 0x046d;
/// HID++ long-report usage page / usage. Filtering on this pair gives us one
/// HID node per physical HID++ device on every supported OS.
const HIDPP_USAGE_PAGE: u16 = 0xff00;
const HIDPP_LONG_USAGE_ID: u16 = 0x0002;

/// How long to wait for device-arrival event bursts before assuming the
/// receiver has finished reporting. 500 ms is comfortably above what a Bolt
/// receiver takes for 6 slots on the hardware I have access to.
const ARRIVAL_DRAIN: Duration = Duration::from_millis(500);

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("HID transport error")]
    Hid(#[from] async_hid::HidError),
}

/// Enumerate all Logitech HID++ receivers visible to the current process and
/// the devices paired to each.
///
/// Devices that are paired but currently offline are not listed: hidpp 0.2 has
/// no public accessor for the wireless PID of an offline pairing, and there is
/// nothing else useful to report about a sleeping device.
pub async fn enumerate() -> Result<Vec<DeviceInventory>, InventoryError> {
    let backend = HidBackend::default();
    let candidates: Vec<async_hid::Device> = backend
        .enumerate()
        .await?
        .filter(|d| {
            d.vendor_id == LOGITECH_VID
                && d.usage_page == HIDPP_USAGE_PAGE
                && d.usage_id == HIDPP_LONG_USAGE_ID
        })
        .collect()
        .await;

    debug!(count = candidates.len(), "HID++ candidate interfaces");

    let mut inventories = Vec::new();
    for dev in candidates {
        match probe_one(dev).await {
            Ok(Some(inv)) => inventories.push(inv),
            Ok(None) => {}
            Err(e) => warn!(error = ?e, "skipping device that failed to probe"),
        }
    }

    Ok(inventories)
}

async fn probe_one(dev: async_hid::Device) -> Result<Option<DeviceInventory>, InventoryError> {
    // `Device: Deref<Target = DeviceInfo>` — clone the deref'd value so we
    // can keep using `dev` (which `to_device_info` would consume).
    let info: async_hid::DeviceInfo = (*dev).clone();
    let (reader, writer) = dev.open().await?;
    let raw = AsyncHidChannel::new(reader, writer, info.clone());

    let channel = match HidppChannel::from_raw_channel(raw).await {
        Ok(c) => Arc::new(c),
        Err(e) => {
            debug!(name = %info.name, error = ?e, "not a HID++ channel");
            return Ok(None);
        }
    };

    let Some(Receiver::Bolt(bolt)) = receiver::detect(Arc::clone(&channel)) else {
        debug!(
            vid = format_args!("{:04x}", info.vendor_id),
            pid = format_args!("{:04x}", info.product_id),
            "no supported receiver (hidpp 0.2 only recognises Logi Bolt)"
        );
        return Ok(None);
    };

    let unique_id = bolt.get_unique_id().await.ok();
    let connections = drain_device_arrival(&bolt).await;

    let mut paired = Vec::with_capacity(connections.len());
    for conn in connections {
        let codename = bolt.get_device_codename(U4::from_lo(conn.index)).await.ok();
        let battery = if conn.online {
            probe_battery(&channel, conn.index).await
        } else {
            None
        };
        paired.push(PairedDevice {
            slot: conn.index,
            codename,
            wpid: Some(conn.wpid),
            kind: map_kind(conn.kind),
            online: conn.online,
            battery,
        });
    }
    paired.sort_by_key(|p| p.slot);

    Ok(Some(DeviceInventory {
        receiver: ReceiverInfo {
            name: "Logi Bolt Receiver".to_string(),
            vendor_id: info.vendor_id,
            product_id: info.product_id,
            unique_id,
        },
        paired,
    }))
}

async fn drain_device_arrival(bolt: &BoltReceiver) -> Vec<BoltDeviceConnection> {
    let rx = bolt.listen();
    if let Err(e) = bolt.trigger_device_arrival().await {
        debug!(error = ?e, "trigger_device_arrival failed; receiver may report no devices");
        return Vec::new();
    }

    let mut out = Vec::new();
    loop {
        match timeout(ARRIVAL_DRAIN, rx.recv()).await {
            Ok(Ok(BoltEvent::DeviceConnection(c))) => out.push(c),
            Ok(Ok(_)) => {} // BoltEvent is non_exhaustive; ignore future variants
            Ok(Err(_)) | Err(_) => break,
        }
    }
    out
}

async fn probe_battery(channel: &Arc<HidppChannel>, slot: u8) -> Option<BatteryInfo> {
    let mut device = match Device::new(Arc::clone(channel), slot).await {
        Ok(d) => d,
        Err(e) => {
            debug!(slot, error = ?e, "Device::new failed");
            return None;
        }
    };
    if let Err(e) = device.enumerate_features().await {
        debug!(slot, error = ?e, "enumerate_features failed");
        return None;
    }
    let feature = device.get_feature::<UnifiedBatteryFeatureV0>()?;
    let info = feature.get_battery_info().await.ok()?;
    Some(BatteryInfo {
        percentage: info.charging_percentage,
        level: map_battery_level(info.level),
        status: map_battery_status(info.status),
    })
}

fn map_kind(k: BoltDeviceKind) -> DeviceKind {
    match k {
        BoltDeviceKind::Keyboard => DeviceKind::Keyboard,
        BoltDeviceKind::Mouse => DeviceKind::Mouse,
        BoltDeviceKind::Numpad => DeviceKind::Numpad,
        BoltDeviceKind::Presenter => DeviceKind::Presenter,
        BoltDeviceKind::Remote => DeviceKind::Remote,
        BoltDeviceKind::Trackball => DeviceKind::Trackball,
        BoltDeviceKind::Touchpad => DeviceKind::Touchpad,
        BoltDeviceKind::Tablet => DeviceKind::Tablet,
        BoltDeviceKind::Gamepad => DeviceKind::Gamepad,
        BoltDeviceKind::Joystick => DeviceKind::Joystick,
        BoltDeviceKind::Headset => DeviceKind::Headset,
        _ => DeviceKind::Unknown,
    }
}

fn map_battery_level(level: HidppBatteryLevel) -> BatteryLevel {
    match level {
        HidppBatteryLevel::Critical => BatteryLevel::Critical,
        HidppBatteryLevel::Low => BatteryLevel::Low,
        HidppBatteryLevel::Good => BatteryLevel::Good,
        HidppBatteryLevel::Full => BatteryLevel::Full,
        _ => BatteryLevel::Unknown,
    }
}

fn map_battery_status(status: HidppBatteryStatus) -> BatteryStatus {
    match status {
        HidppBatteryStatus::Discharging => BatteryStatus::Discharging,
        HidppBatteryStatus::Charging => BatteryStatus::Charging,
        HidppBatteryStatus::ChargingSlow => BatteryStatus::ChargingSlow,
        HidppBatteryStatus::Full => BatteryStatus::Full,
        HidppBatteryStatus::Error => BatteryStatus::Error,
        _ => BatteryStatus::Unknown,
    }
}
