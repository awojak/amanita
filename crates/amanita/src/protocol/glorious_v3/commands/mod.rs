// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Glorious V3 commands and their typed responses.
//!
//! Pass a [`GloriousCommandSpec`](super::protocol::GloriousCommandSpec) with a decoder to
//! [`GloriousV3::read`](super::protocol::GloriousV3::read) for an empty GET request.
//! Use [`GetCommand`](super::protocol::GetCommand) to supply GET parameters.
//! A specification with an encoder can prepare a request for
//! [`GloriousV3::write`](super::protocol::GloriousV3::write) using its `set` method.

mod battery;
mod dpi;
mod status;

use super::protocol::GloriousError;

pub use battery::{BATTERY_INFO, Battery, BatteryInfo, BatteryStatus};
pub use dpi::{DPI_ACTIVE_STAGE, DpiActiveStage};
pub(super) use status::{CommandStatus, LAST_COMMAND_STATUS};

/// Checks that the payload contains exactly `expected` bytes.
///
/// # Errors
///
/// Returns [`GloriousError::InvalidPayloadSize`] if the payload length differs from `expected`.
fn verify_payload_length(payload: &[u8], expected: usize) -> Result<(), GloriousError> {
    let actual = payload.len();
    if actual != expected {
        return Err(GloriousError::InvalidPayloadSize { actual, expected });
    }
    Ok(())
}
