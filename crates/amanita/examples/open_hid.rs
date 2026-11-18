// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{env, error::Error};

use amanita::transport::hid::HidBackend;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os().nth(1).ok_or("usage: open_hid <HID path>")?;
    let mut backend = HidBackend::new()?;
    let endpoint = backend
        .enumerate()?
        .into_iter()
        .find(|endpoint| endpoint.path.as_bytes() == path.as_encoded_bytes())
        .ok_or("HID endpoint not found")?;

    let _connection = backend.open(&endpoint)?;
    println!(
        "Opened {:04x}:{:04x} {}",
        endpoint.vendor_id,
        endpoint.product_id,
        endpoint.path.to_string_lossy()
    );

    Ok(())
}
