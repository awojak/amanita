// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Logical devices and the operations available to the application.

mod battery;
pub mod drivers;

use std::{
    error::Error as StdError,
    ffi::{CStr, OsStr},
};

use thiserror::Error;

pub use battery::{Battery, BatteryInfo, BatteryLevel, BatteryStatus};

use crate::transport::hid::{HidBackend, HidEndpoint, HidError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Battery,
}

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error(transparent)]
    Hid(#[from] HidError),
    #[error(transparent)]
    Protocol(Box<dyn StdError + Send + Sync>),
    #[error("HID endpoint not found")]
    EndpointNotFound,
    #[error("{0:?} is unavailable for this device")]
    Unavailable(Capability),
}

/// Common operations for any device managed by Amanita.
pub trait Device {
    fn name(&self) -> &str;

    fn path(&self) -> &CStr;

    /// Lists the operations implemented by Amanita for this device.
    fn capabilities(&self) -> &[Capability] {
        &[]
    }

    /// # Errors
    /// Returns an error if battery reporting is unavailable or communication fails.
    fn battery(&self, _hid: &HidBackend) -> Result<BatteryInfo, DeviceError> {
        Err(DeviceError::Unavailable(Capability::Battery))
    }
}

/// Common interface for mice. Mouse-specific operations belong on this trait.
pub trait MouseDevice: Device {}

/// Registers a model or family for discovery through a HID endpoint.
pub trait HidDevice: Device + Sized + 'static {
    const VID_PIDS: &'static [(u16, u16)];

    /// Called after the VID/PID matches. Can verify the configuration interface.
    fn from_endpoint(endpoint: &HidEndpoint) -> Option<Self>;
}

type DeviceProbe = fn(&HidEndpoint) -> Option<Box<dyn Device>>;

pub struct DeviceManager {
    hid: HidBackend,
    probes: Vec<DeviceProbe>,
}

impl DeviceManager {
    /// # Errors
    /// Returns an error if HID initialization fails.
    pub fn new() -> Result<Self, DeviceError> {
        let mut manager = Self {
            hid: HidBackend::new()?,
            probes: Vec::new(),
        };
        drivers::register_builtins(&mut manager);
        Ok(manager)
    }

    /// Adds a device implementation to discovery. The first matching registration wins.
    pub fn register<D: HidDevice>(&mut self) {
        self.probes.push(|endpoint| {
            if !D::VID_PIDS.contains(&(endpoint.vendor_id, endpoint.product_id)) {
                return None;
            }
            let device = D::from_endpoint(endpoint)?;
            Some(Box::new(device))
        });
    }

    /// Lists devices identified by automatic discovery.
    /// # Errors
    /// Returns an error if HID enumeration fails.
    pub fn devices(&mut self) -> Result<Vec<Box<dyn Device>>, DeviceError> {
        Ok(self
            .hid
            .enumerate()?
            .iter()
            .filter_map(|endpoint| self.probes.iter().find_map(|probe| probe(endpoint)))
            .collect())
    }

    /// Finds an endpoint for an explicit device selection.
    /// # Errors
    /// Returns an error if HID enumeration fails or the path is absent.
    pub fn endpoint_at(&mut self, path: &OsStr) -> Result<HidEndpoint, DeviceError> {
        self.hid
            .enumerate()?
            .into_iter()
            .find(|endpoint| endpoint.path.as_bytes() == path.as_encoded_bytes())
            .ok_or(DeviceError::EndpointNotFound)
    }

    /// Reads the available batteries for a logical device.
    /// # Errors
    /// Returns an error if battery reporting is unavailable or communication fails.
    pub fn battery(&self, device: &dyn Device) -> Result<BatteryInfo, DeviceError> {
        device.battery(&self.hid)
    }
}
