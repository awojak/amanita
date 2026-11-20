// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Logitech HID++ devices. Frame encoding and request handling belong here.

/// Identifiers for the M705 HID++ 2.x child endpoint exposed by Linux.
pub const M705_VID_PIDS: &[(u16, u16)] = &[(0x046D, 0x406D)];
