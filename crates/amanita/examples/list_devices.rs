// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use amanita::device::{Capability, Device, DeviceError, DeviceManager, drivers::GloriousMouse};

fn main() -> Result<(), DeviceError> {
    let mut manager = DeviceManager::new()?;
    let mut devices = manager.devices()?;
    if let Some(path) = env::args_os().nth(1) {
        let selected = GloriousMouse::new(manager.endpoint_at(&path)?);
        devices.retain(|device| device.path() != selected.path());
        devices.push(Box::new(selected));
    }

    for device in devices {
        println!("{} {}", device.name(), device.path().to_string_lossy());
        println!("  Capabilities: {:?}", device.capabilities());
        if device.capabilities().contains(&Capability::Battery) {
            match manager.battery(device.as_ref()) {
                Ok(battery) => println!("  Battery: {battery:?}"),
                Err(error) => eprintln!("{} battery: {error}", device.name()),
            }
        }
    }

    Ok(())
}
