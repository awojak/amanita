// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryLevel {
    Percent(u8),
    Critical,
    Low,
    Normal,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Charged,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Battery {
    pub level: Option<BatteryLevel>,
    pub status: BatteryStatus,
    pub voltage_mv: Option<u16>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BatteryInfo {
    pub main: Option<Battery>,
    pub backup: Option<Battery>,
}
