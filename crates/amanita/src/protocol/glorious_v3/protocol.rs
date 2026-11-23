// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    thread,
    time::{Duration, Instant},
};

use thiserror::Error;

use super::commands::{CommandStatus, status::CommandStatusCode};
use crate::transport::hid::{HidConnection, HidError};

#[cfg(test)]
#[path = "protocol_test.rs"]
mod tests;

pub(super) const REPORT_ID: u8 = 0x03;
pub(super) const REPORT_SIZE: usize = 64;
pub(super) const GET_BIT: u8 = 0x80;
const MOUSE_ID: u8 = 0xBB;
const HEADER_SIZE: usize = 4;
const RESPONSE_POLL_INTERVAL: Duration = Duration::from_millis(2);
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Error)]
pub enum GloriousError {
    #[error(transparent)]
    Transport(#[from] HidError),
    #[error("invalid length {actual}/{expected} bytes")]
    InvalidLength { actual: usize, expected: usize },
    #[error("response validation failed ({0})")]
    InvalidResponse(&'static str),
    #[error("command execution failed with status {0:#04x}")]
    CommandFailed(u8),
    #[error("transaction timed out")]
    Timeout,
}

/// Identifies a Glorious V3 command.
pub trait GloriousCommand: Sized {
    /// Command ID.
    const ID: u8;
}

/// A command that supports GET.
pub trait GloriousReadCommand: GloriousCommand {
    /// The request argument. Use `()` for a command with an empty request payload.
    type Request;

    /// Encodes the GET request argument as payload bytes.
    fn encode(request: Self::Request) -> impl AsRef<[u8]>;

    /// Decodes the response payload.
    ///
    /// # Errors
    /// Returns an error if the payload is invalid for this command.
    fn decode(payload: &[u8]) -> Result<Self, GloriousError>;
}

/// A command that supports SET.
pub trait GloriousWriteCommand: GloriousCommand {
    /// Encodes the SET payload.
    fn encode(self) -> impl AsRef<[u8]>;
}

fn prepare_get_request<T: GloriousReadCommand>(
    request: T::Request,
) -> Result<[u8; REPORT_SIZE], GloriousError> {
    prepare_request(T::ID, GET_BIT, T::encode(request).as_ref())
}

fn prepare_set_request<T: GloriousWriteCommand>(
    value: T,
) -> Result<[u8; REPORT_SIZE], GloriousError> {
    prepare_request(T::ID, 0, value.encode().as_ref())
}

fn parse_response<T: GloriousReadCommand>(report: &[u8]) -> Result<T, GloriousError> {
    use GloriousError::{InvalidLength, InvalidResponse};

    if report.len() > REPORT_SIZE {
        return Err(InvalidLength {
            actual: report.len(),
            expected: REPORT_SIZE,
        });
    }

    let [
        report_id,
        address,
        command,
        payload_len,
        available_payload @ ..,
    ] = report
    else {
        return Err(InvalidLength {
            actual: report.len(),
            expected: HEADER_SIZE,
        });
    };
    if *report_id != REPORT_ID || *address != MOUSE_ID {
        return Err(InvalidResponse("unexpected report ID or device address"));
    }
    let command_id = normalize_command_id(T::ID);
    // Only the status response requires the GET bit in the echoed command.
    let matches_command = if command_id == normalize_command_id(CommandStatus::ID) {
        *command == command_id | GET_BIT
    } else {
        normalize_command_id(*command) == command_id
    };
    if !matches_command {
        return Err(InvalidResponse("unexpected command"));
    }
    let payload_len = usize::from(*payload_len);
    let payload = available_payload.get(..payload_len).ok_or(InvalidLength {
        actual: available_payload.len(),
        expected: payload_len,
    })?;
    T::decode(payload)
}

/// Returns the command ID with the GET bit cleared.
fn normalize_command_id(command_id: u8) -> u8 {
    command_id & !GET_BIT
}

fn prepare_request(
    command_id: u8,
    operation_bit: u8,
    payload: &[u8],
) -> Result<[u8; REPORT_SIZE], GloriousError> {
    let command_id = normalize_command_id(command_id);
    let payload_len = payload.len();
    let max_payload_len = REPORT_SIZE - HEADER_SIZE;
    if payload_len > max_payload_len {
        return Err(GloriousError::InvalidLength {
            actual: payload_len,
            expected: max_payload_len,
        });
    }

    let mut report = [0; REPORT_SIZE];
    report[..HEADER_SIZE].copy_from_slice(&[
        REPORT_ID,
        MOUSE_ID,
        command_id | operation_bit,
        payload_len as u8,
    ]);
    report[HEADER_SIZE..HEADER_SIZE + payload_len].copy_from_slice(payload);
    Ok(report)
}

/// Blocking protocol connection to a caller-selected mouse configuration endpoint.
///
/// Each transaction shares one timeout across report sending, response polling and
/// the status check. A blocking HID call can outlast the timeout.
///
/// If multiple connections share an endpoint, serialize entire transactions.
/// The device reports the status of its most recent command, so another
/// connection can replace that status before the original caller reads it.
pub struct GloriousV3 {
    connection: HidConnection,
}

impl GloriousV3 {
    pub fn new(connection: HidConnection) -> Self {
        Self { connection }
    }

    /// Encodes the request argument, executes the GET and decodes its response.
    ///
    /// Reading [`CommandStatus`] returns the status of the most recent command.
    /// Pass `()` for commands that have an empty request payload.
    /// For indexed commands, pass the index as the request argument.
    /// Pass `None` to use the default timeout of 500 ms.
    ///
    /// # Errors
    ///
    /// Returns [`GloriousError::InvalidLength`] if the request payload is too large or
    /// a response length is invalid. Other errors include transport failures,
    /// malformed responses, nonzero firmware status, and timeouts.
    pub fn read<T: GloriousReadCommand>(
        &mut self,
        request: T::Request,
        timeout: Option<Duration>,
    ) -> Result<T, GloriousError> {
        let mut report = prepare_get_request::<T>(request)?;

        let transaction_timeout = TransactionTimeout::new(timeout);
        self.send_request(&report, &transaction_timeout)?;
        let received_bytes = self.read_response(&mut report, &transaction_timeout)?;
        if normalize_command_id(T::ID) != CommandStatus::ID {
            self.check_status(T::ID, &transaction_timeout)?;
        }
        parse_response(&report[..received_bytes])
    }

    /// Executes a SET request and checks the device status.
    ///
    /// Pass `None` to use the default timeout of 500 ms.
    ///
    /// # Errors
    /// Returns [`GloriousError::InvalidLength`] if the request payload is too large or
    /// the status response length is invalid. Other errors include transport failures,
    /// malformed status responses, nonzero firmware status, and timeouts.
    pub fn write<T: GloriousWriteCommand>(
        &mut self,
        value: T,
        timeout: Option<Duration>,
    ) -> Result<(), GloriousError> {
        let report = prepare_set_request(value)?;
        let transaction_timeout = TransactionTimeout::new(timeout);
        self.send_request(&report, &transaction_timeout)?;
        self.check_status(T::ID, &transaction_timeout)
    }

    fn check_status(
        &mut self,
        command_id: u8,
        timeout: &TransactionTimeout,
    ) -> Result<(), GloriousError> {
        let mut status_report = prepare_get_request::<CommandStatus>(())?;
        self.send_request(&status_report, timeout)?;
        let received_bytes = self.read_response(&mut status_report, timeout)?;
        let status: CommandStatus = parse_response(&status_report[..received_bytes])?;
        if normalize_command_id(status.command_id) != normalize_command_id(command_id) {
            return Err(GloriousError::InvalidResponse("status belongs to another command"));
        }
        match status.status_code {
            CommandStatusCode::Success => Ok(()),
            CommandStatusCode::Unknown(code) => Err(GloriousError::CommandFailed(code)),
        }
    }

    fn send_request(
        &mut self,
        report: &[u8; REPORT_SIZE],
        timeout: &TransactionTimeout,
    ) -> Result<(), GloriousError> {
        timeout.remaining()?;
        // HidConnection inserts the report ID; send only the remaining 63 bytes.
        self.connection
            .send_feature_report(REPORT_ID, &report[1..])?;
        timeout.remaining()?;
        Ok(())
    }

    fn read_response(
        &mut self,
        report: &mut [u8; REPORT_SIZE],
        timeout: &TransactionTimeout,
    ) -> Result<usize, GloriousError> {
        loop {
            thread::sleep(RESPONSE_POLL_INTERVAL.min(timeout.remaining()?));
            timeout.remaining()?;
            let count = self.connection.get_feature_report(REPORT_ID, report)?;
            timeout.remaining()?;
            let response = report.get(..count).ok_or(GloriousError::InvalidLength {
                actual: count,
                expected: REPORT_SIZE,
            })?;
            if !Self::is_pending_response(response) {
                return Ok(count);
            }
        }
    }

    fn is_pending_response(response: &[u8]) -> bool {
        // Some transports retain the report ID even when no response is ready.
        matches!(response, [0 | REPORT_ID, data @ ..] if data.iter().all(|byte| *byte == 0))
    }
}

struct TransactionTimeout {
    started: Instant,
    duration: Duration,
}

impl TransactionTimeout {
    fn new(timeout: Option<Duration>) -> Self {
        Self {
            started: Instant::now(),
            duration: timeout.unwrap_or(DEFAULT_TIMEOUT),
        }
    }

    fn remaining(&self) -> Result<Duration, GloriousError> {
        let remaining = self.duration.saturating_sub(self.started.elapsed());
        if remaining.is_zero() {
            return Err(GloriousError::Timeout);
        }
        Ok(remaining)
    }
}
