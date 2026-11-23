// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod commands;
pub mod protocol;

use self::protocol::{REPORT_ID, REPORT_SIZE};
use crate::transport::hid::HidEndpoint;

/// USB identifiers of supported Glorious V3 receivers.
pub const VID_PIDS: &[(u16, u16)] = &[(0x3794, 0xA300)];

pub(crate) fn is_configuration_endpoint(endpoint: &HidEndpoint) -> bool {
    endpoint.has_feature_report(REPORT_ID, REPORT_SIZE - 1)
}
