// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::process::Command;

#[test]
fn app_startup_prints_name_and_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_amanita"))
        .output()
        .expect("failed to run amanita");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, format!("Amanita v{}\n", env!("CARGO_PKG_VERSION")).as_bytes());
    assert!(output.stderr.is_empty());
}
