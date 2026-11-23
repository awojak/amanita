// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Glorious V3 commands and their typed responses.
//!
//! The result type selects the command for [`GloriousV3::read`](super::protocol::GloriousV3::read).
//! The value type selects the command for [`GloriousV3::write`](super::protocol::GloriousV3::write).

mod battery;
mod status;

pub use battery::{Battery, BatteryInfo, BatteryStatus};
pub(super) use status::{CommandStatus, LastCommandStatus};
