// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use super::{
    GloriousCommand, GloriousError, GloriousReadCommand, GloriousV3, GloriousWriteCommand,
    TransactionTimeout, parse_response, prepare_get_request, prepare_request, prepare_set_request,
};

const REPORT_LEN: usize = 64;
const HEADER_LEN: usize = 4;
const REPORT_ID: u8 = 0x03;
const MOUSE_ID: u8 = 0xBB;
const GET_BIT: u8 = 0x80;

const CMD_READ_WRITE_ID: u8 = 0x21;
const CMD_READ_ONLY_ID: u8 = 0x23;

fn set_get_bit(command_id: u8) -> u8 {
    command_id | GET_BIT
}

fn build_report(command_id: u8, payload: &[u8]) -> [u8; REPORT_LEN] {
    let payload_len = u8::try_from(payload.len()).unwrap();
    let header: [u8; HEADER_LEN] = [REPORT_ID, MOUSE_ID, command_id, payload_len];
    let mut report = [0; REPORT_LEN];
    report[..HEADER_LEN].copy_from_slice(&header);
    report[HEADER_LEN..HEADER_LEN + payload.len()].copy_from_slice(payload);
    report
}

struct ReadWriteCommand {
    value: u8,
}

impl GloriousCommand for ReadWriteCommand {
    const ID: u8 = CMD_READ_WRITE_ID;
}

impl GloriousReadCommand for ReadWriteCommand {
    type Request = u8;

    fn encode(value: Self::Request) -> impl AsRef<[u8]> {
        [value]
    }

    fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        match payload {
            [value] => Ok(Self { value: *value }),
            _ => Err(GloriousError::InvalidLength {
                actual: payload.len(),
                expected: 1,
            }),
        }
    }
}

impl GloriousWriteCommand for ReadWriteCommand {
    fn encode(self) -> impl AsRef<[u8]> {
        [self.value]
    }
}

struct ReadOnlyCommand;

impl GloriousCommand for ReadOnlyCommand {
    const ID: u8 = CMD_READ_ONLY_ID;
}

impl GloriousReadCommand for ReadOnlyCommand {
    type Request = ();

    fn encode((): Self::Request) -> impl AsRef<[u8]> {
        []
    }

    fn decode(_: &[u8]) -> Result<Self, GloriousError> {
        Ok(Self)
    }
}

#[test]
fn encodes_get_requests() {
    let request = prepare_get_request::<ReadOnlyCommand>(()).unwrap();
    let expected_report = build_report(set_get_bit(CMD_READ_ONLY_ID), &[]);
    assert_eq!(request.as_slice(), &expected_report);

    let payload = 2;
    let request = prepare_get_request::<ReadWriteCommand>(payload).unwrap();
    let expected_report = build_report(set_get_bit(CMD_READ_WRITE_ID), &[payload]);
    assert_eq!(request.as_slice(), &expected_report);
}

#[test]
fn encodes_set_request() {
    let payload = 0x42;
    let command = ReadWriteCommand { value: payload };
    let request = prepare_set_request(command).unwrap();
    let expected_report = build_report(CMD_READ_WRITE_ID, &[payload]);
    assert_eq!(request.as_slice(), &expected_report);
}

#[test]
fn encodes_request_with_maximum_payload() {
    let payload = [0x42; 60];
    let request = prepare_request(CMD_READ_WRITE_ID, GET_BIT, &payload).unwrap();
    let expected_report = build_report(set_get_bit(CMD_READ_WRITE_ID), &payload);
    assert_eq!(request.as_slice(), &expected_report);
}

#[test]
fn rejects_reports_outside_length_bounds() {
    let valid_report = build_report(set_get_bit(CMD_READ_WRITE_ID), &[0x42]);
    let mut oversized_report = [0xFF; REPORT_LEN + 1];
    oversized_report[..REPORT_LEN].copy_from_slice(&valid_report);
    let cases: [(&str, &[u8], usize); 3] = [
        ("empty report", &[], HEADER_LEN),
        ("truncated header", &valid_report[..HEADER_LEN - 1], HEADER_LEN),
        ("oversized report", &oversized_report, REPORT_LEN),
    ];

    for (case, report, expected_len) in cases {
        let result = parse_response::<ReadWriteCommand>(report);
        assert!(
            matches!(
                result,
                Err(GloriousError::InvalidLength { actual, expected })
                    if (actual, expected) == (report.len(), expected_len)
            ),
            "{case}: report {report:02x?}"
        );
    }
}

#[test]
fn rejects_reports_with_unexpected_ids() {
    let payload = [0x42];
    let payload_end_idx = HEADER_LEN + payload.len();
    let valid_report = build_report(set_get_bit(CMD_READ_WRITE_ID), &payload);

    for (case, byte_idx, byte) in [
        ("unexpected report ID", 0, 0x00),
        ("unexpected device address", 1, 0xEE),
        ("unexpected command ID", 2, set_get_bit(CMD_READ_ONLY_ID)),
    ] {
        let mut report = valid_report;
        report[byte_idx] = byte;
        let report = &report[..payload_end_idx];
        let result = parse_response::<ReadWriteCommand>(report);
        assert!(
            matches!(result, Err(GloriousError::InvalidResponse(_))),
            "{case}: report {report:02x?}"
        );
    }
}

#[test]
fn rejects_invalid_response_payload_lengths() {
    let payload = [0x42];
    let payload_end_idx = HEADER_LEN + payload.len();
    let valid_report = build_report(set_get_bit(CMD_READ_WRITE_ID), &payload);
    let payload_len_idx = HEADER_LEN - 1;

    for (case, declared_payload_len, report_end_idx, actual_len, expected_len) in [
        ("missing payload", 1, HEADER_LEN, 0, 1),
        ("zero declared payload length", 0, payload_end_idx, 0, 1),
        ("declared payload exceeds available bytes", 2, payload_end_idx, 1, 2),
    ] {
        let mut report = valid_report;
        report[payload_len_idx] = declared_payload_len;
        let report = &report[..report_end_idx];
        let result = parse_response::<ReadWriteCommand>(report);
        assert!(
            matches!(
                result,
                Err(GloriousError::InvalidLength { actual, expected })
                    if (actual, expected) == (actual_len, expected_len)
            ),
            "{case}: report {report:02x?}"
        );
    }
}

#[test]
fn rejects_oversized_payload() {
    let payload = [0; REPORT_LEN - HEADER_LEN + 1];
    let error = prepare_request(CMD_READ_WRITE_ID, GET_BIT, &payload).unwrap_err();
    assert!(matches!(
        error,
        GloriousError::InvalidLength {
            actual: 61,
            expected: 60,
        }
    ));
}

#[test]
fn uses_default_and_custom_timeouts() {
    let default_timeout = TransactionTimeout::new(None);
    assert_eq!(default_timeout.duration, Duration::from_millis(500));

    let custom_duration = Duration::from_millis(250);
    let custom_timeout = TransactionTimeout::new(Some(custom_duration));
    assert_eq!(custom_timeout.duration, custom_duration);
}

#[test]
fn reports_timeout_when_no_time_remains() {
    let duration = Duration::from_millis(15);
    let expired_timeout = TransactionTimeout {
        started: Instant::now() - duration,
        duration,
    };
    let zero_timeout = TransactionTimeout::new(Some(Duration::ZERO));
    for timeout in [zero_timeout, expired_timeout] {
        let remaining = timeout.remaining();
        let timed_out = matches!(remaining, Err(GloriousError::Timeout));
        assert!(timed_out);
    }
}

#[test]
fn recognizes_pending_reports() {
    let mut report = [0; REPORT_LEN];
    assert!(GloriousV3::is_pending_response(&report));

    report[0] = REPORT_ID;
    assert!(GloriousV3::is_pending_response(&report));

    let payload = [0x42];
    let response = build_report(set_get_bit(CMD_READ_WRITE_ID), &payload);
    assert!(!GloriousV3::is_pending_response(&response));
}
