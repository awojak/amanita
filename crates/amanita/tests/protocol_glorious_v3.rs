// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use amanita::protocol::glorious_v3::{
    commands::{BATTERY_INFO, BatteryInfo},
    protocol::{CommandDesc, REPORT_SIZE},
};

// Synthetic response: main charging at 80%, 4200 mV;
// backup discharging at 42%, 3700 mV.
const BATTERY: [u8; 12] = [0x03, 0xBB, 0x82, 8, 1, 80, 0x10, 0x68, 2, 42, 0x0E, 0x74];

#[test]
fn encodes_get_requests_from_command_descriptions() {
    let request = BATTERY_INFO.get_request();
    assert_eq!(&request[..4], &[0x03, 0xBB, 0x82, 0]);
    assert!(request[4..].iter().all(|byte| *byte == 0));

    let status: CommandDesc<[u8; 2]> = CommandDesc {
        id: 0x7F,
        response_len: 2,
        decode: |payload| Ok([payload[0], payload[1]]),
    };
    assert_eq!(&status.get_request()[..4], &[0x03, 0xBB, 0xFF, 0]);
    assert_eq!(status.parse_response(&[0x03, 0xBB, 0xFF, 2, 2, 0]).unwrap(), [2, 0]);
    assert!(status.parse_response(&[0x03, 0xBB, 0x7F, 2, 2, 0]).is_err());
}

#[test]
fn supports_another_get_with_its_own_payload_decoder() {
    let command = CommandDesc {
        id: 0x10,
        response_len: 1,
        decode: |payload| Ok(payload[0] == 1),
    };

    assert_eq!(&command.get_request()[..4], &[0x03, 0xBB, 0x90, 0]);
    assert!(
        command
            .parse_response(&[0x03, 0xBB, 0x10, 1, 1, 0xFF])
            .unwrap()
    );
    assert!(
        !command
            .parse_response(&[0x03, 0xBB, 0x90, 1, 0, 0xFF])
            .unwrap()
    );
}

#[test]
fn decodes_both_batteries_and_big_endian_voltages() {
    let info = BATTERY_INFO.parse_response(&BATTERY).unwrap();
    assert_eq!((info.main.status, info.main.level_percent, info.main.voltage_mv), (1, 80, 4200));
    assert_eq!(
        (info.backup.status, info.backup.level_percent, info.backup.voltage_mv),
        (2, 42, 3700)
    );
}

#[test]
fn accepts_padding_and_a_response_without_the_get_bit() {
    let mut report = [0; REPORT_SIZE];
    report[..BATTERY.len()].copy_from_slice(&BATTERY);
    report[2] = 0x02;
    assert_eq!(
        BATTERY_INFO.parse_response(&report).unwrap(),
        BatteryInfo::decode(&BATTERY[4..]).unwrap()
    );
}

#[test]
fn preserves_missing_and_unknown_battery_values() {
    let payload = [0, 0xFF, 0, 0, 0xFE, 42, 0x0E, 0x74];
    let info = BatteryInfo::decode(&payload).unwrap();
    assert_eq!(info.main.status, 0);
    assert_eq!(info.main.level_percent, 0xFF);
    assert_eq!(info.backup.status, 0xFE);
}

#[test]
fn rejects_truncated_reports_and_incorrect_headers() {
    for length in 0..BATTERY.len() {
        assert!(BATTERY_INFO.parse_response(&BATTERY[..length]).is_err());
    }
    assert!(BATTERY_INFO.parse_response(&[0; REPORT_SIZE + 1]).is_err());
    for (offset, value) in [(0, 0), (1, 0xAA), (2, 0x83), (3, 7), (3, 9), (3, 255)] {
        let mut report = BATTERY;
        report[offset] = value;
        assert!(BATTERY_INFO.parse_response(&report).is_err());
    }
    assert!(BatteryInfo::decode(&BATTERY[4..11]).is_err());
    assert!(BatteryInfo::decode(&BATTERY[3..]).is_err());
}
