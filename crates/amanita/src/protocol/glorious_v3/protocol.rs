// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{thread, time::Duration};

use thiserror::Error;

use crate::transport::hid::{HidConnection, HidError};

pub const REPORT_ID: u8 = 0x03;
pub const REPORT_SIZE: usize = 64;
const MOUSE_ID: u8 = 0xBB;
const GET_BIT: u8 = 0x80;
const HEADER_SIZE: usize = 4;
const LAST_COMMAND_STATUS: CommandDesc<[u8; 2]> = CommandDesc {
    id: 0x7F,
    response_len: 2,
    decode: |payload| Ok([payload[0], payload[1]]),
};

#[derive(Debug, Error)]
pub enum GloriousError {
    #[error(transparent)]
    Transport(#[from] HidError),
    #[error("invalid Glorious V3 command: {0}")]
    InvalidCommand(&'static str),
    #[error("invalid Glorious V3 response: {0}")]
    InvalidResponse(&'static str),
    #[error("command failed with device status {0:#04x}")]
    CommandFailed(u8),
}

/// A GET command description with a typed payload decoder.
/// Length excludes the header and HID padding.
#[derive(Clone, Copy, Debug)]
pub struct CommandDesc<T> {
    /// Command ID without the GET bit.
    pub id: u8,
    pub response_len: usize,
    pub decode: fn(&[u8]) -> Result<T, GloriousError>,
}

impl<T> CommandDesc<T> {
    /// Encodes an empty GET request including the report ID and zero padding.
    pub fn get_request(&self) -> [u8; REPORT_SIZE] {
        let mut report = [0; REPORT_SIZE];
        report[..HEADER_SIZE].copy_from_slice(&[REPORT_ID, MOUSE_ID, self.id | GET_BIT, 0]);
        report
    }

    /// Validates the response header and decodes its payload, ignoring padding.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid header/length or a payload decoding failure.
    pub fn parse_response(&self, report: &[u8]) -> Result<T, GloriousError> {
        use GloriousError::InvalidResponse;

        if !(HEADER_SIZE..=REPORT_SIZE).contains(&report.len()) {
            return Err(InvalidResponse("invalid report length"));
        }
        if report[0] != REPORT_ID || report[1] != MOUSE_ID {
            return Err(InvalidResponse("unexpected report ID or device address"));
        }
        // Only the status response requires the GET bit in the echoed command.
        let matches_command = if self.id == LAST_COMMAND_STATUS.id {
            report[2] == self.id | GET_BIT
        } else {
            report[2] & !GET_BIT == self.id
        };
        if !matches_command || usize::from(report[3]) != self.response_len {
            return Err(InvalidResponse("unexpected command or payload length"));
        }
        let payload = report
            .get(HEADER_SIZE..HEADER_SIZE + self.response_len)
            .ok_or(InvalidResponse("truncated payload"))?;
        (self.decode)(payload)
    }
}

/// Blocking protocol connection to a caller-selected mouse configuration endpoint.
pub struct GloriousV3 {
    connection: HidConnection,
}

impl GloriousV3 {
    pub fn new(connection: HidConnection) -> Self {
        Self { connection }
    }

    /// Executes an empty GET request, checks its status and decodes the result.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid command, transport failure, invalid
    /// response or nonzero firmware status.
    pub fn get<T>(&mut self, command: CommandDesc<T>) -> Result<T, GloriousError> {
        if command.id & GET_BIT != 0 {
            return Err(GloriousError::InvalidCommand("ID includes the GET bit"));
        }
        if command.response_len > REPORT_SIZE - HEADER_SIZE {
            return Err(GloriousError::InvalidCommand("response payload exceeds the report size"));
        }

        let mut response = command.get_request();
        let response = self.exchange(&mut response, command.response_len)?;
        self.check_status(command.id)?;
        command.parse_response(response)
    }

    fn check_status(&mut self, command_id: u8) -> Result<(), GloriousError> {
        let mut status_report = LAST_COMMAND_STATUS.get_request();
        let status_report = self.exchange(&mut status_report, LAST_COMMAND_STATUS.response_len)?;
        let [last_command, status] = LAST_COMMAND_STATUS.parse_response(status_report)?;
        if last_command & !GET_BIT != command_id {
            return Err(GloriousError::InvalidResponse("status belongs to another command"));
        }
        if status != 0 {
            return Err(GloriousError::CommandFailed(status));
        }
        Ok(())
    }

    // Reuses the request buffer for the received report.
    fn exchange<'a>(
        &mut self,
        report: &'a mut [u8; REPORT_SIZE],
        response_len: usize,
    ) -> Result<&'a [u8], GloriousError> {
        thread::sleep(Duration::from_millis(10));
        // HidConnection inserts the report ID; send only the remaining 63 bytes.
        self.connection
            .send_feature_report(REPORT_ID, &report[1..])?;
        // Radio routing needs 10 ms per meaningful byte plus transaction margins.
        let routing_ms = (HEADER_SIZE * 2 + response_len) as u64 * 10;
        thread::sleep(Duration::from_millis(20 + routing_ms));
        let count = self.connection.get_feature_report(REPORT_ID, report)?;
        thread::sleep(Duration::from_millis(10));
        report
            .get(..count)
            .ok_or(GloriousError::InvalidResponse("invalid report length"))
    }
}
