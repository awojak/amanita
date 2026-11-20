// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Copy, Default)]
struct ReportGlobals {
    id: u32,
    size: u32,
    count: u32,
}

pub(super) fn feature_report_payload_size(mut bytes: &[u8], report_id: u8) -> Option<usize> {
    let mut globals = ReportGlobals::default();
    let mut stack = Vec::new();
    let mut feature_bits: Option<u64> = None;

    while let Some((&prefix, rest)) = bytes.split_first() {
        if prefix == 0xFE {
            // A long item starts with its data length and tag.
            let size = usize::from(*rest.first()?);
            bytes = rest.get(2 + size..)?;
            continue;
        }

        let size = [0, 1, 2, 4][usize::from(prefix & 0x03)];
        let (data, remaining) = rest.split_at_checked(size)?;
        let mut value = [0; 4];
        value[..size].copy_from_slice(data);
        let value = u32::from_le_bytes(value);
        bytes = remaining;

        match prefix & 0xFC {
            0x74 => globals.size = value,
            0x84 => globals.id = value,
            0x94 => globals.count = value,
            0xA4 => stack.push(globals),
            0xB4 => globals = stack.pop()?,
            0xB0 if globals.id == u32::from(report_id) => {
                let bits = u64::from(globals.size) * u64::from(globals.count);
                feature_bits = Some(feature_bits.unwrap_or(0).checked_add(bits)?);
            }
            _ => {}
        }
    }

    usize::try_from(feature_bits?.div_ceil(8)).ok()
}
