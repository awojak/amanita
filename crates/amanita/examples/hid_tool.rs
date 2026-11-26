// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    env,
    error::Error,
    ffi::{OsStr, OsString},
    num::{NonZeroUsize, ParseIntError},
    thread,
    time::Duration,
};

use amanita::transport::hid::{HidBackend, HidError};

const SEND_REPORT_SIZE: usize = 64;
const RAW_EXCHANGE_DELAY: Duration = Duration::from_millis(200);

fn usage() -> String {
    format!(
        "Usage: cargo run --example hid_tool -- <operation>

  list
  raw-get <HID path> <report ID> <read size>
  raw-set <HID path> <report ID> [payload bytes...]
  raw-exchange <HID path> <report ID> <read size> [payload bytes...]

Read size is decimal and includes the report ID byte.
Report IDs and payload bytes accept decimal or 0x-prefixed hexadecimal.
Outgoing reports are padded with zeros to {SEND_REPORT_SIZE} bytes, including the report ID.
raw-exchange waits {} ms before reading.\n",
        RAW_EXCHANGE_DELAY.as_millis(),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let operation = arguments.next().unwrap_or_else(|| "list".into());
    let arguments: Vec<_> = arguments.collect();
    match (operation.to_str(), arguments.as_slice()) {
        (Some("-h" | "--help"), []) => print!("{}", usage()),
        (Some("list"), []) => list_endpoints()?,
        (Some("raw-get"), [path, report_id, size]) => transfer(path, report_id, Some(size), None)?,
        (Some("raw-set"), [path, report_id, bytes @ ..]) => {
            transfer(path, report_id, None, Some(bytes))?;
        }
        (Some("raw-exchange"), [path, report_id, size, bytes @ ..]) => {
            transfer(path, report_id, Some(size), Some(bytes))?;
        }
        _ => {
            eprint!("{}", usage());
            return Err("invalid arguments".into());
        }
    }
    Ok(())
}

fn list_endpoints() -> Result<(), HidError> {
    for endpoint in HidBackend::new()?.enumerate()? {
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

fn transfer(
    path: &OsStr,
    report_id: &OsStr,
    read_size: Option<&OsStr>,
    payload_bytes: Option<&[OsString]>,
) -> Result<(), Box<dyn Error>> {
    let report_id = parse_byte(report_id)?;
    let read_buffer = if let Some(size) = read_size {
        let size = size.to_string_lossy().parse::<NonZeroUsize>()?.get();
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(size)?;
        buffer.resize(size, 0);
        Some(buffer)
    } else {
        None
    };
    let payload = if let Some(bytes) = payload_bytes {
        let mut payload = [0; SEND_REPORT_SIZE - 1];
        let data = payload
            .get_mut(..bytes.len())
            .ok_or("payload exceeds report capacity")?;
        for (byte, argument) in data.iter_mut().zip(bytes) {
            *byte = parse_byte(argument)?;
        }
        Some(payload)
    } else {
        None
    };

    let mut backend = HidBackend::new()?;
    let endpoint = backend
        .enumerate()?
        .into_iter()
        .find(|endpoint| endpoint.path.as_bytes() == path.as_encoded_bytes())
        .ok_or("HID endpoint not found")?;
    let mut connection = backend.open(&endpoint)?;
    if let Some(payload) = payload {
        connection.send_feature_report(report_id, &payload)?;
        if read_buffer.is_some() {
            thread::sleep(RAW_EXCHANGE_DELAY);
        } else {
            println!("Sent feature report {report_id:#04x} ({SEND_REPORT_SIZE} bytes)");
        }
    }
    if let Some(mut buffer) = read_buffer {
        let count = connection.get_feature_report(report_id, &mut buffer)?;
        let response = buffer
            .get(..count)
            .ok_or("HID report exceeds the buffer size")?;
        println!("{response:02x?}");
    }
    Ok(())
}

fn parse_byte(argument: &OsStr) -> Result<u8, ParseIntError> {
    let value = argument.to_string_lossy();
    match value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        Some(digits) => u8::from_str_radix(digits, 16),
        None => value.parse(),
    }
}
