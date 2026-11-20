// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use amanita::protocol::glorious_v3::protocol::{GloriousCommandSpec, REPORT_SIZE};

const COMMAND: GloriousCommandSpec<bool> = GloriousCommandSpec {
    id: 0x10,
    decode: Some(|payload| Ok(payload == [1])),
    encode: None,
};

const RESPONSE: [u8; 5] = [0x03, 0xBB, 0x90, 1, 1];

#[test]
fn encodes_and_decodes_a_get_command() {
    assert_eq!(&COMMAND.get_request().unwrap()[..4], &[0x03, 0xBB, 0x90, 0]);
    assert!(COMMAND.parse_response(&RESPONSE).unwrap());
    assert!(!COMMAND.parse_response(&[0x03, 0xBB, 0x90, 1, 0]).unwrap());
}

#[test]
fn accepts_padding_and_a_response_without_the_get_bit() {
    let mut report = [0xFF; REPORT_SIZE];
    report[..RESPONSE.len()].copy_from_slice(&RESPONSE);
    report[2] = 0x10;
    assert!(COMMAND.parse_response(&report).unwrap());
}

#[test]
fn rejects_truncated_reports_and_incorrect_headers() {
    for length in 0..RESPONSE.len() {
        assert!(COMMAND.parse_response(&RESPONSE[..length]).is_err());
    }

    let mut oversized = [0; REPORT_SIZE + 1];
    oversized[..RESPONSE.len()].copy_from_slice(&RESPONSE);
    assert!(COMMAND.parse_response(&oversized).is_err());

    for (offset, value) in [(0, 0), (1, 0xAA), (2, 0x91), (3, 2), (3, 255)] {
        let mut report = RESPONSE;
        report[offset] = value;
        assert!(COMMAND.parse_response(&report).is_err());
    }
}
