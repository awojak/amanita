// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{GloriousCommandSpec, GloriousError};
use super::verify_payload_length;

/// GET command that returns a [`BatteryInfo`].
pub const BATTERY_INFO: GloriousCommandSpec<BatteryInfo> = GloriousCommandSpec {
    id: 0x02,
    decode: Some(BatteryInfo::decode),
    encode: None,
};

/// Battery presence and charging state.
///
/// [`BatteryStatus::Unknown`] preserves the original byte for unrecognized codes.
pub enum BatteryStatus {
    Missing,
    Charging,
    Discharging,
    Charged,
    Unknown(u8),
}

/// Status, charge level, and voltage for one battery.
///
/// The charge level and voltage are undefined when the status is [`BatteryStatus::Missing`].
pub struct Battery {
    pub status: BatteryStatus,
    pub level_percent: u8,
    pub voltage_mv: u16,
}

impl Battery {
    /// Decodes a raw battery record into a `Battery`.
    fn decode([status, level_percent, voltage_high, voltage_low]: [u8; 4]) -> Self {
        let status = match status {
            0 => BatteryStatus::Missing,
            1 => BatteryStatus::Charging,
            2 => BatteryStatus::Discharging,
            3 => BatteryStatus::Charged,
            value => BatteryStatus::Unknown(value),
        };
        Self {
            status,
            level_percent,
            voltage_mv: u16::from_be_bytes([voltage_high, voltage_low]),
        }
    }
}

/// Main and backup battery readings returned by [`BATTERY_INFO`].
pub struct BatteryInfo {
    pub main: Battery,
    pub backup: Battery,
}

impl BatteryInfo {
    /// Decodes an eight-byte payload into a `BatteryInfo`.
    ///
    /// # Errors
    ///
    /// Returns [`GloriousError::InvalidPayloadSize`] if the payload length is invalid.
    pub fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        verify_payload_length(payload, 8)?;
        let (records, _) = payload.as_chunks::<4>();
        Ok(Self {
            main: Battery::decode(records[0]),
            backup: Battery::decode(records[1]),
        })
    }
}
