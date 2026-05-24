use anyhow::{Context, Result};
use clap::Args;
use openlogi_core::device::{BatteryInfo, DeviceInventory, PairedDevice};

#[derive(Debug, Args)]
pub struct ListArgs {}

pub async fn run(_args: ListArgs) -> Result<()> {
    let inventories = openlogi_hid::enumerate()
        .await
        .context("failed to enumerate HID++ devices")?;

    if inventories.is_empty() {
        println!("No Logitech HID++ receivers found.");
        println!();
        println!("Notes:");
        println!("  - On macOS, quit Logi Options+ first — both apps fight over HID++ access.");
        println!("  - hidpp 0.2 only recognises Logi Bolt receivers (PID 0xC548).");
        println!("  - Devices paired directly over Bluetooth are not enumerated yet.");
        std::process::exit(2);
    }

    for (i, inv) in inventories.iter().enumerate() {
        if i != 0 {
            println!();
        }
        print_inventory(inv);
    }

    Ok(())
}

fn print_inventory(inv: &DeviceInventory) {
    let uid = inv.receiver.unique_id.as_deref().unwrap_or("—");
    println!(
        "{} ({}, vid={:04x} pid={:04x})",
        inv.receiver.name, uid, inv.receiver.vendor_id, inv.receiver.product_id
    );

    if inv.paired.is_empty() {
        println!("  └─ no paired devices");
        return;
    }

    let last = inv.paired.len() - 1;
    for (i, d) in inv.paired.iter().enumerate() {
        let prefix = if i == last { "  └─" } else { "  ├─" };
        println!("{prefix} {}", format_device(d));
    }
}

fn format_device(d: &PairedDevice) -> String {
    let dot = if d.online { "●" } else { "○" };
    let codename = d.codename.as_deref().unwrap_or("Unknown device");
    let wpid = d
        .wpid
        .map_or_else(|| "wpid=?".to_string(), |w| format!("wpid={w:04x}"));
    let battery = d
        .battery
        .as_ref()
        .map_or_else(|| "battery=—".to_string(), format_battery);
    let kind = format!("{:?}", d.kind).to_lowercase();
    format!(
        "slot {} {dot} {codename} ({kind}, {wpid}, {battery})",
        d.slot
    )
}

fn format_battery(b: &BatteryInfo) -> String {
    let level = format!("{:?}", b.level).to_lowercase();
    let status = format!("{:?}", b.status).to_lowercase();
    format!("battery={}% {level} ({status})", b.percentage)
}
