// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commands for communicating with Glorious V3 devices.
//!
//! - To read data, choose the response type `T` when calling [`GloriousV3::read`].
//!   `read` sends the command associated with `T` and decodes the reply into `T`.
//! - To write data, pass a value to [`GloriousV3::write`]. `write` uses the value's
//!   type to choose the command and encodes the value as the request payload.
//!
//! [`GloriousV3::read`]: super::protocol::GloriousV3::read
//! [`GloriousV3::write`]: super::protocol::GloriousV3::write

pub mod battery;
mod dpi;
pub mod status;

pub use battery::BatteryInfo;
pub use dpi::DpiStage;
pub use status::CommandStatus;
