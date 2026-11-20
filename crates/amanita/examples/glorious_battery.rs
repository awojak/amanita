// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{env, error::Error};

use amanita::{
    protocol::glorious_v3::{
        commands::{BATTERY_INFO, Battery, BatteryStatus},
        protocol::GloriousV3,
    },
    transport::hid::HidBackend,
};

fn print_battery(name: &str, battery: &Battery) {
    let status = match &battery.status {
        BatteryStatus::Missing => {
            println!("{name}: missing");
            return;
        }
        BatteryStatus::Charging => "charging",
        BatteryStatus::Discharging => "discharging",
        BatteryStatus::Charged => "charged",
        BatteryStatus::Unknown(value) => {
            println!("{name}: unknown status {value:#04x}");
            return;
        }
    };
    println!("{name}: {status}, {}%, {} mV", battery.level_percent, battery.voltage_mv);
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os()
        .nth(1)
        .ok_or("usage: glorious_battery <HID configuration path>")?;
    let mut backend = HidBackend::new()?;
    let endpoint = backend
        .enumerate()?
        .into_iter()
        .find(|endpoint| endpoint.path.as_bytes() == path.as_encoded_bytes())
        .ok_or("HID endpoint not found")?;
    let mut protocol = GloriousV3::new(backend.open(&endpoint)?);
    let info = protocol.read(BATTERY_INFO)?;
    print_battery("Main battery", &info.main);
    print_battery("Backup battery", &info.backup);
    Ok(())
}
