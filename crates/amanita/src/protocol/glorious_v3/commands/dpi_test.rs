// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{DpiStage, GloriousError, GloriousReadCommand, GloriousWriteCommand};

#[test]
fn encodes_and_decodes_stages() {
    for stage in u8::MIN..=u8::MAX {
        let new_dpi_stage = DpiStage { stage };
        let encoded_dpi_stage = new_dpi_stage.encode();
        assert_eq!(encoded_dpi_stage.as_ref(), &[stage]);

        let active_dpi_stage = DpiStage::decode(encoded_dpi_stage.as_ref()).unwrap();
        assert_eq!(active_dpi_stage, new_dpi_stage);
    }
}

#[test]
fn rejects_invalid_payload_lengths() {
    let invalid_payloads: &[&[u8]] = &[&[], &[0, 0]];
    for &payload in invalid_payloads {
        let result = DpiStage::decode(payload);
        assert!(matches!(
            result,
            Err(GloriousError::InvalidLength { actual, expected: 1 })
                if actual == payload.len()
        ));
    }
}
