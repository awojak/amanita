// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use crate::device::{
    Battery, BatteryInfo, BatteryLevel, BatteryStatus, Capability, Device, DeviceError, HidDevice,
    MouseDevice,
};
use crate::protocol::glorious_v3::{self, commands, protocol::GloriousV3};
use crate::transport::hid::{HidBackend, HidEndpoint};

pub struct GloriousMouse {
    endpoint: HidEndpoint,
}

impl GloriousMouse {
    /// Uses a caller-selected Glorious V3 configuration endpoint.
    pub fn new(endpoint: HidEndpoint) -> Self {
        Self { endpoint }
    }
}

impl Device for GloriousMouse {
    fn name(&self) -> &str {
        "Glorious V3 mouse"
    }

    fn path(&self) -> &CStr {
        &self.endpoint.path
    }

    fn capabilities(&self) -> &[Capability] {
        &[Capability::Battery]
    }

    fn battery(&self, hid: &HidBackend) -> Result<BatteryInfo, DeviceError> {
        let mut protocol = GloriousV3::new(hid.open(&self.endpoint)?);
        let battery_info: commands::BatteryInfo = protocol
            .read((), None)
            .map_err(|error| DeviceError::Protocol(Box::new(error)))?;
        Ok(BatteryInfo {
            main: convert_battery(battery_info.main),
            backup: convert_battery(battery_info.backup),
        })
    }
}

impl MouseDevice for GloriousMouse {}

impl HidDevice for GloriousMouse {
    const VID_PIDS: &'static [(u16, u16)] = glorious_v3::VID_PIDS;

    fn from_endpoint(endpoint: &HidEndpoint) -> Option<Self> {
        glorious_v3::is_configuration_endpoint(endpoint).then(|| Self::new(endpoint.clone()))
    }
}

fn convert_battery(battery: commands::Battery) -> Option<Battery> {
    let status = match battery.status {
        commands::BatteryStatus::Missing => return None,
        commands::BatteryStatus::Charging => BatteryStatus::Charging,
        commands::BatteryStatus::Discharging => BatteryStatus::Discharging,
        commands::BatteryStatus::Charged => BatteryStatus::Charged,
        commands::BatteryStatus::Unknown(_) => BatteryStatus::Unknown,
    };
    Some(Battery {
        level: (battery.level_percent <= 100)
            .then_some(BatteryLevel::Percent(battery.level_percent)),
        status,
        voltage_mv: Some(battery.voltage_mv),
    })
}
