// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CString;

use amanita::transport::hid::{HidBackend, HidEndpoint, HidError};

#[test]
fn rejects_non_utf8_path() {
    let backend = HidBackend::new().unwrap();
    let endpoint = HidEndpoint {
        path: CString::new(b"/dev/hidraw\xff".to_vec()).unwrap(),
        vendor_id: 0,
        product_id: 0,
        serial_number: None,
    };

    let error = backend.open(&endpoint).unwrap_err();

    assert!(matches!(error, HidError::InvalidPath(_)));
}

#[test]
fn rejects_missing_path() {
    let backend = HidBackend::new().unwrap();

    let endpoint = HidEndpoint {
        path: CString::new(b"/dev/missing-device".to_vec()).unwrap(),
        vendor_id: 0,
        product_id: 0,
        serial_number: None,
    };

    let error = backend.open(&endpoint).unwrap_err();

    assert!(matches!(error, HidError::HidApi(_)));
}
