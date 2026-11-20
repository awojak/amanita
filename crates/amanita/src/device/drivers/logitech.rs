// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use crate::device::{Device, HidDevice, MouseDevice};
use crate::protocol::logitech::hidpp;
use crate::transport::hid::HidEndpoint;

pub struct LogitechM705 {
    endpoint: HidEndpoint,
}

impl Device for LogitechM705 {
    fn name(&self) -> &str {
        "Logitech M705"
    }

    fn path(&self) -> &CStr {
        &self.endpoint.path
    }
}

impl MouseDevice for LogitechM705 {}

impl HidDevice for LogitechM705 {
    const VID_PIDS: &'static [(u16, u16)] = hidpp::M705_VID_PIDS;

    fn from_endpoint(endpoint: &HidEndpoint) -> Option<Self> {
        Some(Self {
            endpoint: endpoint.clone(),
        })
    }
}
