// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{GloriousCommand, GloriousError, GloriousReadCommand};

#[cfg(test)]
#[path = "status_test.rs"]
mod tests;

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

impl GloriousCommand for LastCommandStatus {
    const ID: u8 = 0x7F;
}

impl GloriousReadCommand for LastCommandStatus {
    type Request = ();

    fn encode((): Self::Request) -> impl AsRef<[u8]> {
        []
    }

    fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        let &[command_id, code] = payload else {
            return Err(GloriousError::InvalidLength {
                actual: payload.len(),
                expected: 2,
            });
        };
        let status_code = match code {
            0 => CommandStatus::Success,
            code => CommandStatus::Unknown(code),
        };
        Ok(Self {
            command_id,
            status_code,
        })
    }
}
