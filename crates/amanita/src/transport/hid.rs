// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

mod descriptor;

use std::{collections::HashSet, ffi::CString, fs, path::Path, str::Utf8Error};

use hidapi::{HidApi, HidDevice, HidError as HidApiError};
use thiserror::Error;

#[cfg(test)]
#[path = "hid_test.rs"]
mod tests;

/// An error in HID operations.
#[derive(Debug, Error)]
pub enum HidError {
    #[error("invalid buffer size")]
    InvalidBufferSize,

    #[error("HID path is not valid UTF-8: {0}")]
    InvalidPath(#[source] Utf8Error),

    #[error("{0}")]
    HidApi(#[from] HidApiError),
}

/// A HID endpoint identified by its path.
#[derive(Clone)]
pub struct HidEndpoint {
    pub path: CString,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
}

impl HidEndpoint {
    /// Checks the Linux HID descriptor for a feature report with the given payload size.
    /// The payload size excludes the report ID byte.
    pub fn has_feature_report(&self, report_id: u8, payload_size: usize) -> bool {
        let Ok(path) = self.path.to_str() else {
            return false;
        };
        let Some(hidraw_name) = Path::new(path).file_name() else {
            return false;
        };
        let descriptor_path = Path::new("/sys/class/hidraw")
            .join(hidraw_name)
            .join("device/report_descriptor");
        fs::read(descriptor_path).is_ok_and(|bytes| {
            descriptor::feature_report_payload_size(&bytes, report_id) == Some(payload_size)
        })
    }
}

/// Enumerates and opens HID endpoints.
pub struct HidBackend {
    api: HidApi,
}

impl HidBackend {
    /// Creates a new HID backend.
    ///
    /// # Errors
    ///
    /// Returns an error if HIDAPI initialization fails.
    pub fn new() -> Result<Self, HidError> {
        Ok(Self {
            api: HidApi::new()?,
        })
    }

    /// Enumerates HID endpoints.
    ///
    /// Refreshes the endpoint list and returns one entry per unique HID path.
    /// Endpoints with identical vendor IDs, product IDs, or serial numbers
    /// are included if their paths differ.
    ///
    /// # Errors
    ///
    /// Returns an error if refreshing the endpoint list fails.
    pub fn enumerate(&mut self) -> Result<Vec<HidEndpoint>, HidError> {
        self.api.refresh_devices()?;

        let mut paths = HashSet::new();
        let endpoints = self
            .api
            .device_list()
            .filter(|device| paths.insert(device.path()))
            .map(|device| HidEndpoint {
                path: device.path().to_owned(),
                vendor_id: device.vendor_id(),
                product_id: device.product_id(),
                serial_number: device.serial_number().map(str::to_owned),
            })
            .collect();

        Ok(endpoints)
    }

    /// Opens a HID endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The HID endpoint to open, identified by its `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not valid UTF-8 or the endpoint cannot be opened.
    pub fn open(&self, endpoint: &HidEndpoint) -> Result<HidConnection, HidError> {
        endpoint.path.to_str().map_err(HidError::InvalidPath)?;

        Ok(HidConnection {
            device: self.api.open_path(&endpoint.path)?,
        })
    }
}

/// A handle to an open HID endpoint.
///
/// The handle is automatically closed when the connection is dropped.
#[derive(Debug)]
pub struct HidConnection {
    device: HidDevice,
}

impl HidConnection {
    /// Sends a feature report.
    ///
    /// This is a blocking operation.
    ///
    /// # Arguments
    ///
    /// * `report_id` - Report ID to send.
    /// * `payload` - Data without the report ID, including any required padding.
    ///   May be empty.
    ///
    /// # Errors
    ///
    /// Returns an error if sending fails.
    pub fn send_feature_report(&mut self, report_id: u8, payload: &[u8]) -> Result<(), HidError> {
        let mut report = Vec::with_capacity(payload.len() + 1);
        report.push(report_id);
        report.extend_from_slice(payload);

        self.device.send_feature_report(&report)?;
        Ok(())
    }

    /// Receives a feature report.
    ///
    /// This is a blocking operation.
    ///
    /// # Arguments
    ///
    /// * `report_id` - Report ID to request.
    /// * `buffer` - Destination for the report ID and data. Must be at least 1 byte long.
    ///
    /// # Returns
    ///
    /// Byte count including the report ID at `buffer[0]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is empty or receiving fails.
    pub fn get_feature_report(
        &mut self,
        report_id: u8,
        buffer: &mut [u8],
    ) -> Result<usize, HidError> {
        if buffer.is_empty() {
            return Err(HidError::InvalidBufferSize);
        }

        buffer.fill(0);
        buffer[0] = report_id;
        Ok(self.device.get_feature_report(buffer)?)
    }
}
