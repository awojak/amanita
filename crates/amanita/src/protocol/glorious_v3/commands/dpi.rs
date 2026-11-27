// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt;

use super::super::protocol::{
    GloriousCommand, GloriousError, GloriousReadCommand, GloriousWriteCommand,
};

#[cfg(test)]
#[path = "dpi_test.rs"]
mod tests;

/// The active DPI stage number.
#[derive(Debug)]
pub struct DpiStage {
    pub stage: u8,
}

impl fmt::Display for DpiStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DPI stage {}", self.stage)
    }
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
    fn build(args: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        const EXPECTED_ARGUMENT: &str = "expected argument <stage> as a DPI stage number";

        let [stage] = args else {
            return Err(EXPECTED_ARGUMENT.into());
        };

        let stage = stage
            .parse()
            .map_err(|error| format!("{EXPECTED_ARGUMENT}. {error}"))?;
        Ok(Self { stage })
    }

    fn encode(self) -> impl AsRef<[u8]> {
        [self.stage]
    }
}
