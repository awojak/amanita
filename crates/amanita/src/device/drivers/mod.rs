// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

mod glorious;
mod logitech;

pub use glorious::GloriousMouse;
pub use logitech::LogitechM705;

use super::DeviceManager;

pub(super) fn register_builtins(manager: &mut DeviceManager) {
    manager.register::<GloriousMouse>();
    manager.register::<LogitechM705>();
}
