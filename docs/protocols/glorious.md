# Glorious protocol notes

Status: planning evidence, not a tested device implementation.

## Evidence and scope

The owner supplied two documentation screenshots (SET_REPORT and GET_REPORT)
and an accompanying description on 2026-11-16. The description identifies vendor
Feature Reports with ID `0x03` and states that the USB radio receiver forwards
reports to the mouse for the same processing as direct USB. The owner separately
confirmed that BLE uses the same mechanism.

Plan one common protocol codec for the USB receiver and BLE. Direct wired USB is
mentioned by the supplied documentation but is not an initial implementation
requirement. The exact mouse model, firmware and source document title/revision
remain unspecified; these notes must not be generalized to all Glorious devices.

## USB control requests in the supplied tables

| Field | SET_REPORT | GET_REPORT | Interpretation |
| --- | --- | --- | --- |
| `bmRequestType` | `0x21` | `0xA1` | Class/interface request, host-to-device or device-to-host |
| `bRequest` | `0x09` | `0x01` | SET_REPORT or GET_REPORT |
| `wValue` | `0x0303` | `0x0303` | High byte: Feature type `0x03`; low byte: Report ID `0x03` |
| `wIndex` | `0x0001` | `0x0001` | Interface 1 in the documented USB configuration |
| `wLength` | `0x0040` | `0x0040` | 64-byte USB data stage as written |
| Data | Command-dependent | Command-dependent | Battery Info layout supplied below |

Decoded USB setup packets (16-bit fields are little-endian on the wire):

```text
SET_REPORT: 21 09 03 03 01 00 40 00
GET_REPORT: A1 01 03 03 01 00 40 00
```

These bytes describe the USB setup stage, not an HIDAPI application buffer.
The requests are class-specific even though the report contents are vendor-defined.
The USB fields do not describe the BLE wire framing.

## HIDAPI mapping

Use `send_feature_report` and `get_feature_report` in the Rust HIDAPI binding.
Both take a buffer whose first byte is Report ID `0x03`; command data follows.
HIDAPI and the OS construct the transport request, so the application must not
prepend the eight USB setup bytes. Select the correct enumerated HID path using
device identity, interface and descriptor information. Do not assume that USB
interface number 1 is also meaningful for BLE.

HIDAPI buffer length includes the Report ID byte. Keep the API buffer length,
USB data-stage length and command payload length explicit and separate until the
device descriptor and an observed transaction establish their relationship.

The initial USB tables label data positions `8–72` (65 positions inclusive), but
specify `wLength = 0x0040` (64). The subsequently supplied Battery Info table
explicitly indexes the report `0–63`, including Report ID at byte 0. Plan a
64-byte HIDAPI buffer for this command, with no additional Report ID prefix;
verify it against the descriptor and actual transaction in M1.

## First feature: Battery Info

The owner supplied section `1.4.6 0x02 - Battery Info` and selected this as the
first functional feature, after the foundation PR. Its on-wire Command ID is
`0x82`; do not substitute the section's `0x02` value or infer other command IDs.

Request, 64 bytes including Report ID:

| Byte(s) | Field | Value |
| --- | --- | --- |
| 0 | Report ID | `0x03` |
| 1 | Device ID | `0xBB` |
| 2 | Command ID | `0x82` |
| 3 | Data size | `0` |
| 4–63 | Padding | Zero |

Expected response layout:

| Byte(s) | Field | Documented value |
| --- | --- | --- |
| 0 | Report ID | `0x03` |
| 1 | Device ID | `0xBB` |
| 2 | Command ID | `0x82` |
| 3 | Data size | `6` |
| 4 | Main battery status | `0` missing, `1` charging, `2` discharging, `3` charged |
| 5 | Main battery level | `0–100` percent |
| 6 | Main battery voltage | `0–4500` mV; encoding unresolved |
| 7 | Backup battery status | Same status codes as main battery |
| 8 | Backup battery level | `0–100` percent |
| 9 | Backup battery voltage | `0–4500` mV; encoding unresolved |
| 10–63 | Padding | Zero |

The voltage fields occupy one byte each, which cannot directly encode `0–4500`
mV. Obtain the scaling/encoding or a corrected layout; do not assume a 16-bit
field, shift following offsets or invent a conversion. Voltage can remain unknown
while status and percentage are implemented. Missing backup battery is a valid
state, not evidence of a second battery in the selected mouse.

Plan to send the query through SET_REPORT and retrieve its response through
GET_REPORT, then verify ordering, timing and correlation on hardware. Tests should
cover the request bytes and padding, all documented statuses, absent batteries,
invalid percentage, unknown status, incorrect header/data size and truncated
responses. Report lengths and valid response handling must follow observed device
behavior. No protocol code or test implementation belongs in PR 1.

## M1 validation and missing command details

1. Record model, firmware, USB receiver identifiers and BLE identity.
2. Inspect descriptors and identify the configuration HID path in each mode.
3. Resolve report and payload sizes, including Report ID placement.
4. Use the supplied Battery Info query and validate SET/GET sequencing, response
   correlation, status fields, padding, integrity fields and timing if applicable.
   A SET_REPORT can carry a query; it does not necessarily mean a settings write.
5. Validate the battery/charging layout above; later record DPI, lighting and polling-rate command formats and
   capabilities. Do not invent command bytes or assume all features exist in BLE.
6. Test disconnected/sleeping mouse behavior behind an attached receiver and
   BLE reconnection. Store sanitized request/response fixtures for codec tests.

The supplied evidence defines transport requests and the Battery Info command.
Settings commands have not been supplied. PR 1 remains build-system work only.

## API references

- [HIDAPI API documentation](https://libusb.info/hidapi/group__API.html)
- [Rust HIDAPI binding source and API comments](https://docs.rs/hidapi/latest/src/hidapi/lib.rs.html)
- [Linux hidraw documentation](https://docs.kernel.org/hid/hidraw.html)
