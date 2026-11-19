// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Glorious V3 GET commands and their typed responses.
//!
//! Add a GET command by defining a [`CommandDesc`] with its wire ID (without
//! the GET bit), payload length and decoder. Pass it to
//! [`GloriousV3::get`](super::protocol::GloriousV3::get) to execute it.

use super::protocol::{CommandDesc, GloriousError};

pub const BATTERY_INFO: CommandDesc<BatteryInfo> = CommandDesc {
    id: 0x02,
    response_len: 8,
    decode: BatteryInfo::decode,
};

/// Raw firmware values. Level and voltage are undefined for a missing battery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Battery {
    /// 0 missing, 1 charging, 2 discharging, 3 charged; other values are reserved.
    pub status: u8,
    pub level_percent: u8,
    pub voltage_mv: u16,
}

impl Battery {
    fn decode([status, level_percent, voltage_high, voltage_low]: [u8; 4]) -> Self {
        Self {
            status,
            level_percent,
            voltage_mv: u16::from_be_bytes([voltage_high, voltage_low]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatteryInfo {
    pub main: Battery,
    pub backup: Battery,
}

impl BatteryInfo {
    /// Decodes the eight-byte Battery Info payload, without the protocol header.
    ///
    /// # Errors
    ///
    /// Returns an error if the payload is not eight bytes long.
    pub fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        let ([main, backup], []) = payload.as_chunks::<4>() else {
            return Err(GloriousError::InvalidResponse("expected eight battery bytes"));
        };
        Ok(Self {
            main: Battery::decode(*main),
            backup: Battery::decode(*backup),
        })
    }
}
