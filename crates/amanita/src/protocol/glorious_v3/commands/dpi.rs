// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::protocol::{
    GloriousCommand, GloriousError, GloriousReadCommand, GloriousWriteCommand,
};

#[cfg(test)]
#[path = "dpi_test.rs"]
mod tests;

/// The active DPI stage number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DpiStage {
    pub stage: u8,
}

impl GloriousCommand for DpiStage {
    const ID: u8 = 0x10;
}

impl GloriousReadCommand for DpiStage {
    type Request = ();

    fn encode((): Self::Request) -> impl AsRef<[u8]> {
        []
    }

    fn decode(payload: &[u8]) -> Result<Self, GloriousError> {
        let &[stage] = payload else {
            return Err(GloriousError::InvalidLength {
                actual: payload.len(),
                expected: 1,
            });
        };
        Ok(Self { stage })
    }
}

impl GloriousWriteCommand for DpiStage {
    fn encode(self) -> impl AsRef<[u8]> {
        [self.stage]
    }
}
