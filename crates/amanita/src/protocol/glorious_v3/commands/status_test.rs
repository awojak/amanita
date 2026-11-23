// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{CommandStatus, CommandStatusCode, GloriousError, GloriousReadCommand};
use crate::protocol::glorious_v3::protocol::GET_BIT;

const CMD_TEST_ID: u8 = 0x10;
const SUCCESS_STATUS: u8 = 0;

#[test]
fn decodes_success() {
    let get_command_id = CMD_TEST_ID | GET_BIT;
    for reported_command_id in [CMD_TEST_ID, get_command_id] {
        let payload = [reported_command_id, SUCCESS_STATUS];
        let status = CommandStatus::decode(&payload).unwrap();
        assert_eq!(status.command_id, reported_command_id);
        assert!(matches!(status.status_code, CommandStatusCode::Success));
    }
}

#[test]
fn preserves_unknown_status_codes() {
    let get_command_id = CMD_TEST_ID | GET_BIT;
    for status_code in SUCCESS_STATUS + 1..=u8::MAX {
        let payload = [get_command_id, status_code];
        let status = CommandStatus::decode(&payload).unwrap();
        assert_eq!(status.command_id, get_command_id);
        assert!(matches!(
            status.status_code,
            CommandStatusCode::Unknown(code) if code == status_code
        ));
    }
}

#[test]
fn rejects_invalid_payload_lengths() {
    let oversized_payload = [CMD_TEST_ID, SUCCESS_STATUS, u8::MAX];
    let invalid_payloads: &[&[u8]] = &[&[], &[CMD_TEST_ID], &oversized_payload];
    for &payload in invalid_payloads {
        let result = CommandStatus::decode(payload);
        assert!(matches!(
            result,
            Err(GloriousError::InvalidLength { actual, expected: 2 })
                if actual == payload.len()
        ));
    }
}
