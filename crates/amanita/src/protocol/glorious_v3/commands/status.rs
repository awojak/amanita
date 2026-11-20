// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{GloriousCommandSpec, GloriousError};
use super::verify_payload_length;

/// GET command that returns a [`LastCommandStatus`].
pub(in super::super) const LAST_COMMAND_STATUS: GloriousCommandSpec<LastCommandStatus> =
    GloriousCommandSpec {
        id: 0x7F,
        decode: Some(decode_status),
        encode: None,
    };

/// Result of the most recent command.
pub(in super::super) struct LastCommandStatus {
    /// ID of the command this status refers to. It may include the GET bit.
    pub command_id: u8,
    /// Result of the command.
    pub status_code: CommandStatus,
}

/// Status code returned for a command.
pub(in super::super) enum CommandStatus {
    Success,
    /// An unrecognized code preserved for error reporting.
    Unknown(u8),
}

/// Decodes a two-byte payload into a [`LastCommandStatus`].
///
/// # Errors
///
/// Returns [`GloriousError::InvalidPayloadSize`] if the payload length is invalid.
fn decode_status(payload: &[u8]) -> Result<LastCommandStatus, GloriousError> {
    verify_payload_length(payload, 2)?;
    let status_code = match payload[1] {
        0 => CommandStatus::Success,
        code => CommandStatus::Unknown(code),
    };
    Ok(LastCommandStatus {
        command_id: payload[0],
        status_code,
    })
}
