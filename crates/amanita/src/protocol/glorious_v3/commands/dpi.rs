// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{GloriousCommandSpec, GloriousError};
use super::verify_payload_length;

/// GET and SET command for the active DPI stage.
pub const DPI_ACTIVE_STAGE: GloriousCommandSpec<DpiActiveStage> = GloriousCommandSpec {
    id: 0x10,
    decode: Some(decode_active_stage),
    encode: Some(encode_active_stage),
};

/// The active DPI stage number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DpiActiveStage {
    pub stage: u8,
}

fn decode_active_stage(payload: &[u8]) -> Result<DpiActiveStage, GloriousError> {
    verify_payload_length(payload, 1)?;
    Ok(DpiActiveStage { stage: payload[0] })
}

fn encode_active_stage(DpiActiveStage { stage }: DpiActiveStage) -> Vec<u8> {
    vec![stage]
}
