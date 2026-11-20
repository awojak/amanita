// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use amanita::protocol::glorious_v3::{
    commands::{DPI_ACTIVE_STAGE, DpiActiveStage},
    protocol::{GloriousError, SetCommand},
};

#[test]
fn encodes_the_active_dpi_stage_commands() {
    let get = DPI_ACTIVE_STAGE.get_request().unwrap();
    assert_eq!(&get[..4], &[0x03, 0xBB, 0x90, 0]);
    assert!(get[4..].iter().all(|byte| *byte == 0));

    for stage in [0, 3, 4, u8::MAX] {
        let set = DPI_ACTIVE_STAGE.set(DpiActiveStage { stage }).unwrap();
        assert_eq!(set.id(), 0x10);
        assert_eq!(set.payload().as_ref(), &[stage]);
    }
}

#[test]
fn decodes_the_reported_stage_and_requires_one_payload_byte() {
    for stage in [0, 3, 4, u8::MAX] {
        let report = [0x03, 0xBB, 0x90, 1, stage];
        assert_eq!(DPI_ACTIVE_STAGE.parse_response(&report).unwrap(), DpiActiveStage { stage });
    }
    for report in [[0x03, 0xBB, 0x90, 0, 0, 0], [0x03, 0xBB, 0x90, 2, 0, 0]] {
        assert!(matches!(
            DPI_ACTIVE_STAGE.parse_response(&report),
            Err(GloriousError::InvalidPayloadSize { expected: 1, .. })
        ));
    }
}
