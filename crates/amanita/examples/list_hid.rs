// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use amanita::transport::hid::{HidBackend, HidError};

fn main() -> Result<(), HidError> {
    let mut backend = HidBackend::new()?;

    for endpoint in backend.enumerate()? {
        println!(
            "{:04x}:{:04x} {} serial={}",
            endpoint.vendor_id,
            endpoint.product_id,
            endpoint.path.to_string_lossy(),
            endpoint.serial_number.as_deref().unwrap_or("unknown"),
        );
    }

    Ok(())
}
