// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BatteryInfo, BatteryStatus, GloriousError};

// Synthetic payload: main charging at 80%, 4200 mV;
// backup discharging at 42%, 3700 mV.
const BATTERY_PAYLOAD: [u8; 8] = [1, 80, 0x10, 0x68, 2, 42, 0x0E, 0x74];

#[test]
fn decodes_both_batteries_and_big_endian_voltages() {
    let battery_info = BatteryInfo::decode(&BATTERY_PAYLOAD).unwrap();
    let main = battery_info.main;
    let main_is_charging = matches!(main.status, BatteryStatus::Charging);
    assert!(main_is_charging);
    assert_eq!((main.level_percent, main.voltage_mv), (80, 4200));

    let backup = battery_info.backup;
    let backup_is_discharging = matches!(backup.status, BatteryStatus::Discharging);
    assert!(backup_is_discharging);
    assert_eq!((backup.level_percent, backup.voltage_mv), (42, 3700));
}

#[test]
fn preserves_missing_and_unknown_battery_values() {
    let payload = [0, 0xFF, 0, 0, 0xFE, 42, 0x0E, 0x74];
    let info = BatteryInfo::decode(&payload).unwrap();
    assert!(matches!(info.main.status, BatteryStatus::Missing));
    assert_eq!(info.main.level_percent, 0xFF);
    assert!(matches!(info.backup.status, BatteryStatus::Unknown(0xFE)));
}

#[test]
fn decodes_charged_battery_status() {
    let info = BatteryInfo::decode(&[3, 100, 0x10, 0x68, 0, 0, 0, 0]).unwrap();
    assert!(matches!(info.main.status, BatteryStatus::Charged));
}

#[test]
fn rejects_invalid_payload_lengths() {
    let expected_length = BATTERY_PAYLOAD.len();
    let payload = [0; BATTERY_PAYLOAD.len() + 1];

    for length in (0..expected_length).chain([payload.len()]) {
        let Err(error) = BatteryInfo::decode(&payload[..length]) else {
            panic!("accepted payload of {length} bytes");
        };
        assert!(matches!(
            error,
            GloriousError::InvalidLength { actual, expected }
                if (actual, expected) == (length, expected_length)
        ));
        assert_eq!(error.to_string(), format!("invalid length {length}/{expected_length} bytes"));
    }
}
