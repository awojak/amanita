// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{GloriousCommand, GloriousError, GloriousReadCommand};

#[cfg(test)]
#[path = "status_test.rs"]
mod tests;

/// Status response for the most recent command.
pub struct CommandStatus {
    /// ID of the command this status refers to. It may include the GET bit.
    pub command_id: u8,
    /// Status code reported by the device.
    pub status_code: CommandStatusCode,
}

/// Status code returned for a command.
pub enum CommandStatusCode {
    Success,
    /// An unrecognized code preserved for error reporting.
    Unknown(u8),
}

impl GloriousCommand for CommandStatus {
    const ID: u8 = 0x7F;
}

impl GloriousReadCommand for CommandStatus {
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
            0 => CommandStatusCode::Success,
            code => CommandStatusCode::Unknown(code),
        };
        Ok(Self {
            command_id,
            status_code,
        })
    }
}
