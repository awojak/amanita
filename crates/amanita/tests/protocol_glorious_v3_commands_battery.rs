// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use amanita::protocol::glorious_v3::{
    commands::{BATTERY_INFO, BatteryInfo, BatteryStatus},
    protocol::{GloriousError, REPORT_SIZE},
};

// Synthetic response: main charging at 80%, 4200 mV;
// backup discharging at 42%, 3700 mV.
const BATTERY: [u8; 12] = [0x03, 0xBB, 0x82, 8, 1, 80, 0x10, 0x68, 2, 42, 0x0E, 0x74];

#[test]
fn encodes_the_battery_get_request() {
    let request = BATTERY_INFO.get_request().unwrap();
    assert_eq!(&request[..4], &[0x03, 0xBB, 0x82, 0]);
    assert!(request[4..].iter().all(|byte| *byte == 0));
}

#[test]
fn decodes_both_batteries_and_big_endian_voltages() {
    let info = BATTERY_INFO.parse_response(&BATTERY).unwrap();
    assert!(matches!(
        (info.main.status, info.main.level_percent, info.main.voltage_mv),
        (BatteryStatus::Charging, 80, 4200)
    ));
    assert!(matches!(
        (info.backup.status, info.backup.level_percent, info.backup.voltage_mv),
        (BatteryStatus::Discharging, 42, 3700)
    ));
}

#[test]
fn battery_response_requires_eight_declared_payload_bytes() {
    let mut report = [0xFF; REPORT_SIZE];
    report[..BATTERY.len()].copy_from_slice(&BATTERY);

    let info = BATTERY_INFO.parse_response(&report).unwrap();
    assert_eq!((info.main.voltage_mv, info.backup.voltage_mv), (4200, 3700));

    for length in [7, 9] {
        report[3] = length;
        assert!(matches!(
            BATTERY_INFO.parse_response(&report),
            Err(GloriousError::InvalidPayloadSize { actual, expected: 8 })
                if actual == usize::from(length)
        ));
    }
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
fn battery_decoder_requires_exactly_eight_bytes() {
    let mut payload = [0xFF; REPORT_SIZE];
    payload[..8].copy_from_slice(&BATTERY[4..]);

    for length in (0..8).chain(9..=payload.len()) {
        assert!(matches!(
            BatteryInfo::decode(&payload[..length]),
            Err(GloriousError::InvalidPayloadSize { actual, expected: 8 }) if actual == length
        ));
    }
    let info = BatteryInfo::decode(&payload[..8]).unwrap();
    assert_eq!((info.main.voltage_mv, info.backup.voltage_mv), (4200, 3700));
}
