// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{GloriousCommand, GloriousError, GloriousReadCommand};

#[cfg(test)]
#[path = "battery_test.rs"]
mod tests;

// Status and charge level each occupy one byte. Voltage occupies two bytes.
const BATTERY_RECORD_BYTES: usize = 4;

/// Battery presence and charging state.
///
/// [`BatteryStatus::Unknown`] preserves the original byte for unrecognized codes.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct BatteryReading {
    pub status: BatteryStatus,
    pub level_percent: u8,
    pub voltage_mv: u16,
}

impl BatteryReading {
    /// Decodes one battery record.
    fn decode(
        [status, level_percent, voltage_high, voltage_low]: [u8; BATTERY_RECORD_BYTES],
    ) -> Self {
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

/// Main and backup battery readings.
#[derive(Debug)]
pub struct BatteryInfo {
    pub main: BatteryReading,
    pub backup: BatteryReading,
}

impl BatteryInfo {
    /// Decodes an eight-byte payload into a `BatteryInfo`.
    ///
    /// # Errors
    ///
    /// Returns [`GloriousError::InvalidLength`] if the payload length differs from eight bytes.
    pub fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        let ([main, backup], []) = payload.as_chunks::<BATTERY_RECORD_BYTES>() else {
            return Err(GloriousError::InvalidLength {
                actual: payload.len(),
                expected: 2 * BATTERY_RECORD_BYTES,
            });
        };
        Ok(Self {
            main: BatteryReading::decode(*main),
            backup: BatteryReading::decode(*backup),
        })
    }
}

impl GloriousCommand for BatteryInfo {
    const ID: u8 = 0x02;
}

impl GloriousReadCommand for BatteryInfo {
    type Request = ();

    fn encode((): Self::Request) -> impl AsRef<[u8]> {
        []
    }

    fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        Self::decode(payload)
    }
}
