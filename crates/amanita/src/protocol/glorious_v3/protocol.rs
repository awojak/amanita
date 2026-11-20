// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    thread,
    time::{Duration, Instant},
};

use thiserror::Error;

use super::commands::{CommandStatus, LAST_COMMAND_STATUS};
use crate::transport::hid::{HidConnection, HidError};

pub const REPORT_ID: u8 = 0x03;
pub const REPORT_SIZE: usize = 64;
const MOUSE_ID: u8 = 0xBB;
const GET_BIT: u8 = 0x80;
const HEADER_SIZE: usize = 4;
const RESPONSE_POLL_INTERVAL: Duration = Duration::from_millis(2);
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Error)]
pub enum GloriousError {
    #[error(transparent)]
    Transport(#[from] HidError),
    #[error("invalid Glorious V3 command: {0}")]
    InvalidCommand(&'static str),
    #[error("invalid Glorious V3 response: {0}")]
    InvalidResponse(&'static str),
    #[error("invalid response payload size {actual}/{expected} bytes")]
    InvalidPayloadSize { actual: usize, expected: usize },
    #[error("command failed with device status {0:#04x}")]
    CommandFailed(u8),
    #[error("Glorious V3 transaction timed out")]
    Timeout,
}

/// A GET request with a typed response and optional request data.
pub trait GetCommand {
    type Response;

    /// Describes the command ID and response decoder.
    fn descriptor(&self) -> &GloriousCommandSpec<Self::Response>;

    /// Returns the encoded request data.
    fn payload(&self) -> impl AsRef<[u8]> {
        []
    }
}

/// A SET request with encoded payload data.
pub trait SetCommand {
    /// Command ID without the GET bit.
    fn id(&self) -> u8;

    /// Returns the encoded request data.
    fn payload(&self) -> impl AsRef<[u8]>;
}

pub type Decode<T> = fn(&[u8]) -> Result<T, GloriousError>;
pub type Encode<T> = fn(T) -> Vec<u8>;

/// Describes a command and its payload codecs. A missing codec disables its operation.
#[derive(Clone, Copy, Debug)]
pub struct GloriousCommandSpec<T> {
    /// Command ID without the GET bit.
    pub id: u8,
    pub decode: Option<Decode<T>>,
    pub encode: Option<Encode<T>>,
}

impl<T> GloriousCommandSpec<T> {
    /// Encodes an empty GET request including the report ID and zero padding.
    ///
    /// # Errors
    /// Returns an error if GET is unavailable or the command ID is invalid.
    pub fn get_request(&self) -> Result<[u8; REPORT_SIZE], GloriousError> {
        self.prepare_get_request(&[])
    }

    fn prepare_get_request(&self, payload: &[u8]) -> Result<[u8; REPORT_SIZE], GloriousError> {
        if self.decode.is_none() {
            return Err(GloriousError::InvalidCommand("GET is unavailable for this command"));
        }
        prepare_request(self.id, GET_BIT, payload)
    }

    /// Validates the response header and decodes its payload, ignoring padding.
    ///
    /// # Errors
    ///
    /// Returns an error if GET is unavailable, the response is invalid or decoding fails.
    pub fn parse_response(&self, report: &[u8]) -> Result<T, GloriousError> {
        use GloriousError::InvalidResponse;

        let decode = self
            .decode
            .ok_or(GloriousError::InvalidCommand("GET is unavailable for this command"))?;
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
        if !matches_command {
            return Err(InvalidResponse("unexpected command"));
        }
        let payload_len = usize::from(report[3]);
        let payload = report
            .get(HEADER_SIZE..HEADER_SIZE + payload_len)
            .ok_or(InvalidResponse("truncated payload"))?;
        decode(payload)
    }

    /// Encodes a value as a SET request for this command ID.
    ///
    /// # Errors
    /// Returns an error if SET is unavailable for the command.
    pub fn set(&self, value: T) -> Result<SetRequest, GloriousError> {
        let encode = self
            .encode
            .ok_or(GloriousError::InvalidCommand("SET is unavailable for this command"))?;
        Ok(SetRequest {
            id: self.id,
            payload: encode(value),
        })
    }
}

impl<T> GetCommand for GloriousCommandSpec<T> {
    type Response = T;

    fn descriptor(&self) -> &GloriousCommandSpec<T> {
        self
    }
}

/// A SET request prepared by a command specification.
pub struct SetRequest {
    id: u8,
    payload: Vec<u8>,
}

impl SetCommand for SetRequest {
    fn id(&self) -> u8 {
        self.id
    }

    fn payload(&self) -> impl AsRef<[u8]> {
        self.payload.as_slice()
    }
}

fn prepare_request(
    command_id: u8,
    operation_bit: u8,
    payload: &[u8],
) -> Result<[u8; REPORT_SIZE], GloriousError> {
    if command_id & GET_BIT != 0 {
        return Err(GloriousError::InvalidCommand("ID includes the GET bit"));
    }
    if payload.len() > REPORT_SIZE - HEADER_SIZE {
        return Err(GloriousError::InvalidCommand("request payload exceeds the report size"));
    }

    let mut report = [0; REPORT_SIZE];
    report[..HEADER_SIZE].copy_from_slice(&[
        REPORT_ID,
        MOUSE_ID,
        command_id | operation_bit,
        payload.len() as u8,
    ]);
    report[HEADER_SIZE..HEADER_SIZE + payload.len()].copy_from_slice(payload);
    Ok(report)
}

/// Blocking protocol connection to a caller-selected mouse configuration endpoint.
pub struct GloriousV3 {
    connection: HidConnection,
    timeout: Duration,
}

impl GloriousV3 {
    pub fn new(connection: HidConnection) -> Self {
        Self {
            connection,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Overrides the default two-second timeout for each GET or SET transaction.
    ///
    /// The timeout covers sending, response polling and the status check.
    /// Blocking HID feature report calls can only be checked after they return.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Executes a GET request, checks its status and decodes the result.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid command, transport failure, invalid
    /// response, nonzero firmware status or timeout.
    pub fn read<T>(&mut self, command: impl GetCommand<Response = T>) -> Result<T, GloriousError> {
        let descriptor = command.descriptor();
        let mut report = descriptor.prepare_get_request(command.payload().as_ref())?;

        let timeout = TransactionTimeout::new(self.timeout);
        let response = self.exchange(&mut report, &timeout)?;
        self.check_status(descriptor.id, &timeout)?;
        descriptor.parse_response(response)
    }

    /// Executes a SET request and checks the device status.
    ///
    /// # Errors
    /// Returns an error for an invalid command, transport failure, invalid
    /// status response, nonzero firmware status or timeout.
    pub fn write(&mut self, command: impl SetCommand) -> Result<(), GloriousError> {
        let id = command.id();
        let report = prepare_request(id, 0, command.payload().as_ref())?;
        let timeout = TransactionTimeout::new(self.timeout);
        self.send_request(&report, &timeout)?;
        self.check_status(id, &timeout)
    }

    fn check_status(
        &mut self,
        command_id: u8,
        timeout: &TransactionTimeout,
    ) -> Result<(), GloriousError> {
        let mut status_report = LAST_COMMAND_STATUS.get_request()?;
        let status_report = self.exchange(&mut status_report, timeout)?;
        let status = LAST_COMMAND_STATUS.parse_response(status_report)?;
        if status.command_id & !GET_BIT != command_id {
            return Err(GloriousError::InvalidResponse("status belongs to another command"));
        }
        match status.status_code {
            CommandStatus::Success => Ok(()),
            CommandStatus::Unknown(code) => Err(GloriousError::CommandFailed(code)),
        }
    }

    // Reuses the request buffer for the received report.
    fn exchange<'a>(
        &mut self,
        report: &'a mut [u8; REPORT_SIZE],
        timeout: &TransactionTimeout,
    ) -> Result<&'a [u8], GloriousError> {
        self.send_request(report, timeout)?;
        let count = read_response(report, timeout, |buffer| {
            self.connection.get_feature_report(REPORT_ID, buffer)
        })?;
        Ok(&report[..count])
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
}

struct TransactionTimeout {
    started: Instant,
    duration: Duration,
}

impl TransactionTimeout {
    fn new(duration: Duration) -> Self {
        Self {
            started: Instant::now(),
            duration,
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

fn read_response(
    report: &mut [u8; REPORT_SIZE],
    timeout: &TransactionTimeout,
    mut read: impl FnMut(&mut [u8]) -> Result<usize, HidError>,
) -> Result<usize, GloriousError> {
    loop {
        thread::sleep(RESPONSE_POLL_INTERVAL.min(timeout.remaining()?));
        timeout.remaining()?;
        let count = read(report)?;
        timeout.remaining()?;
        let response = report
            .get(..count)
            .ok_or(GloriousError::InvalidResponse("invalid report length"))?;
        // Some transports retain the report ID even when no response is ready.
        if let [0 | REPORT_ID, data @ ..] = response
            && data.iter().all(|byte| *byte == 0)
        {
            continue;
        }
        return Ok(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::glorious_v3::commands::{DPI_ACTIVE_STAGE, DpiActiveStage};

    #[test]
    fn encodes_get_with_request_data_and_set_from_a_command_object() {
        let get = prepare_request(0x0F, GET_BIT, &[4]).unwrap();
        assert_eq!(&get[..5], &[0x03, 0xBB, 0x8F, 1, 4]);
        assert_eq!(get[5..], [0; REPORT_SIZE - 5]);

        let set_only = GloriousCommandSpec {
            decode: None,
            ..DPI_ACTIVE_STAGE
        };
        assert!(matches!(set_only.get_request(), Err(GloriousError::InvalidCommand(_))));
        let get_only = GloriousCommandSpec {
            encode: None,
            ..DPI_ACTIVE_STAGE
        };
        let stage = DpiActiveStage { stage: 4 };
        assert!(matches!(get_only.set(stage), Err(GloriousError::InvalidCommand(_))));

        for command in [
            DPI_ACTIVE_STAGE.set(stage).unwrap(),
            set_only.set(stage).unwrap(),
        ] {
            let set = prepare_request(command.id(), 0, command.payload().as_ref()).unwrap();
            assert_eq!(&set[..5], &[0x03, 0xBB, 0x10, 1, 4]);
            assert_eq!(set[5..], [0; REPORT_SIZE - 5]);
        }
    }

    #[test]
    fn rejects_a_prefixed_id_and_an_oversized_request_payload() {
        assert!(matches!(prepare_request(0x90, 0, &[]), Err(GloriousError::InvalidCommand(_))));
        let payload = [0; REPORT_SIZE - HEADER_SIZE + 1];
        assert!(matches!(
            prepare_request(0x10, GET_BIT, &payload),
            Err(GloriousError::InvalidCommand(_))
        ));
    }

    #[test]
    fn polls_both_forms_of_zero_report_then_returns_the_response() {
        let mut responses = [[0; REPORT_SIZE]; 3];
        responses[1][0] = REPORT_ID;
        responses[2][..5].copy_from_slice(&[REPORT_ID, MOUSE_ID, 0x90, 1, 0]);
        let mut responses = responses.into_iter();
        let mut report = [0; REPORT_SIZE];
        let mut last_poll = Instant::now();

        let count =
            read_response(&mut report, &TransactionTimeout::new(DEFAULT_TIMEOUT), |buffer| {
                assert!(last_poll.elapsed() >= RESPONSE_POLL_INTERVAL);
                last_poll = Instant::now();
                buffer.copy_from_slice(&responses.next().unwrap());
                Ok(REPORT_SIZE)
            })
            .unwrap();

        assert_eq!(&report[..5], &[REPORT_ID, MOUSE_ID, 0x90, 1, 0]);
        assert_eq!(count, REPORT_SIZE);
        assert_eq!(responses.len(), 0);
    }

    #[test]
    fn times_out_if_only_zero_reports_arrive() {
        let mut report = [0; REPORT_SIZE];
        let timeout = TransactionTimeout::new(Duration::from_millis(15));
        let result = read_response(&mut report, &timeout, |buffer| {
            buffer.fill(0);
            Ok(REPORT_SIZE)
        });

        assert!(matches!(result, Err(GloriousError::Timeout)));
    }
}
