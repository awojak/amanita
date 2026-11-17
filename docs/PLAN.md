# Amanita development plan

Status: project foundation prepared; application architecture below is proposed.

## Product scope

Amanita is a Linux-first manager and configurator for gaming mice and keyboards.
The first supported device will be one specific Glorious mouse, connected through
its USB wireless receiver or Bluetooth LE. The exact model, firmware and device
identifiers still need to be recorded before hardware support begins.

Initial device features are battery and charging status, DPI, lighting effects and
colors, and polling rate, where supported by the device and connection mode.
Button mapping and additional devices follow later. Android is an exploratory
target, not a requirement for the first release.

## Architecture

- `amanita-core`: platform-independent device identities, capabilities, settings,
  battery state, validation and errors. No Qt, D-Bus or Linux dependencies.
- `amanita-hid`: HIDAPI transport, discovery and report exchange. On Linux, prefer
  the hidraw backend to cover USB and Bluetooth HID. Keep transport separate from
  model-specific packet encoding.
- `amanita-glorious`: protocol implementation for the selected model. Record
  supported firmware, interfaces, report IDs and capabilities per connection mode.
- `amanitad`: Rust service owning device access, reconnect handling, settings and
  battery monitoring. Start with a systemd user service and narrowly scoped device
  permissions; validate Bluetooth and USB access rules on actual hardware.
- `amanita-desktop`: session integration for notifications, independent of the GUI
  window lifecycle. Can initially run within the user service.
- Frontend: proposed Qt 6 / Qt Quick interface, using a Linux D-Bus client adapter.
  Choose and validate the Rust/Qt bridge in a separate prototype before committing
  to its build dependencies. A small diagnostic CLI can exercise the same API first.

Only introduce application crates when their implementation starts. PR 1 contains
one `amanita` build target with an empty `main()`; it implements no application
behavior. The core, HID backend, service and frontend belong to later PRs.

The service exposes a versioned session D-Bus API for device enumeration,
capabilities, settings and battery changes. Clients must handle unavailable devices,
unsupported settings and service restarts. The service validates writes. Keep the
API adapter outside the core so Android can eventually use an in-process adapter.
If shared access across simultaneous Linux sessions becomes necessary, revisit
system-service ownership and authorization in an architecture decision.

### Transport verification

HIDAPI is the initial transport for the USB receiver and BLE HID devices. The
owner confirmed that both modes use the same vendor Feature Report mechanism,
with Report ID `0x03`. Share the protocol codec across these modes; keep discovery
and connection handling transport-aware. On Linux, use the hidraw backend and
verify the actual report descriptor, report sizes and feature availability in each
mode. A separate proprietary GATT implementation is not currently planned.

See [Glorious protocol notes](protocols/glorious.md) for the supplied USB request
fields, their HIDAPI mapping and the report-length ambiguity to resolve in M1.

Distinguish a connected receiver from a connected, awake mouse. Identify the
configuration interface using descriptors and usage information as well as VID/PID.
Keep hardware writes serialized, bounded by timeouts and restricted to identified
devices. Do not persist settings repeatedly without checking device semantics.

### Desktop integration

Use the freedesktop notification interface for low-battery and fully-charged
events. Emit on state transitions with deduplication; unknown battery state must
remain unknown. Notifications must work without an open configuration window.

Native battery display in KDE/GNOME is a separate integration task. First inspect
whether the kernel already exposes a power_supply device and UPower reports it.
If only vendor reports provide battery state, investigate an upstream kernel or
UPower integration path. The application publishing its own D-Bus property does
not automatically add a battery to desktop panels. Do not promise native panel
display until this path is demonstrated on the selected hardware.

### Android direction

Qt can provide a shared UI, but Linux D-Bus, systemd, hidraw and udev are platform
adapters. An Android proof of concept must separately validate USB host permissions,
receiver access, BLE/GATT access, background lifecycle and Rust/Qt packaging.
Keep the core portable now; defer Android builds and CI until that investigation.

## Milestones and acceptance criteria

### M0 / PR 1: reproducible project foundation

Deliverables:

- Cargo workspace with Rust edition 2024 and explicit resolver 3; shared package
  metadata and one empty executable target for build verification. No core library,
  service implementation, command handling or hardware functionality.
- Commit `Cargo.lock`. Pin Rust 1.100.0, the latest stable release verified during
  foundation work, including rustfmt and clippy. Make Nix, local development and
  CI read the same `rust-toolchain.toml`.
- Nix flake with locked inputs and `devShells.default`, providing tools and
  dependencies. Use a pinned Rust overlay reading the toolchain file. Keep build,
  check, test and package commands in a small Makefile, with Cargo managing Rust
  compilation and dependencies. Initially
  validate x86_64-linux; add aarch64-linux when it has a working validation path.
- Dev shell with Cargo, rustc, rustfmt, clippy, rust-analyzer and rust-src, plus
  Git, GNU Make, tar, gzip, nixfmt, statix,
  deadnix, actionlint and ShellCheck. Use the
  same compiler for builds and development. Add native HID/Qt dependencies when used.
- GitHub Actions on pull requests and pushes to the default branch: formatting,
  clippy with warnings denied, tests, a Cargo release build, nixfmt/statix/deadnix
  and actionlint. Run Make targets in the locked Nix environment as the common
  toolchain source. Pin third-party actions
  to verified commit SHAs, grant read-only repository permissions, and cancel
  superseded runs. Cache is optional and must not require secrets for fork PRs.
- Use one CI job with sequential `prepare`, `check`, `unit test`, `build`,
  `smoke test`, `package` and `deploy` stages. Share the runner's Nix store within the
  run. Do not add an inter-run cache.
  Use `x86_64-unknown-linux-musl` as the default Cargo target and statically link
  both debug and release executables. Provide the Rust target through
  `rust-toolchain.toml`, using the standard native C linker for the Rust-only
  scaffold. Run Cargo tests and Clippy for the same target. Archive
  Cargo's release output directly with a Git version (tag, commit distance,
  commit ID and optional dirty suffix), and retain it as a GitHub Actions artifact
  for one day, without publishing a release. Fetch full Git history and tags
  in CI. The current scaffold runs without Nix or shared runtime libraries.
  Smoke-test the release build in an empty root filesystem with a five-second
  timeout and a forced stop one second later, failing on a nonzero exit status.
  Revisit packaging and supported distribution testing when native HID or Qt
  dependencies are added.
- README explaining early development status, currently unsupported hardware,
  project goals, planned architecture, verified build/run/check commands and the
  milestone roadmap. Keep repository documentation in English initially.
- Keep Cargo build outputs and package archives in the ignored
  `output/` directory. Ignore local IDE files. Decide the project
  license before distributing code; do not infer a license from dependency choices.

Acceptance from a clean checkout with Nix installed and flakes enabled:

```sh
nix develop --no-update-lock-file -c make check
nix develop --no-update-lock-file -c make test
nix develop --no-update-lock-file -c make build-debug
nix develop --no-update-lock-file -c make package
```

There are no unit tests in PR 1 because there is no application behavior. Prepare
the Cargo test execution path without introducing artificial tests or features.
Document any local check blocked by the environment; GitHub CI results are only
claimed after the workflow actually runs. PR 1 has no HID communication, functional
IPC, GUI, desktop integration, system installation or hardware support claims.

### M1: hardware investigation and read-only backend

Battery Info is the first functional feature. Record the exact mouse model,
firmware, USB receiver VID/PID, BLE identity and
report descriptors. Verify discovery, permissions, read-only protocol access,
battery and charging information in both modes. Capture sanitized protocol
fixtures and model capabilities. Implement and test the Battery Info request and
response codec described in the protocol notes, resolving the voltage encoding
before displaying voltage. Test disconnect, receiver-only and sleep states.
Document any feature unavailable over BLE rather than blocking all USB progress.

### M2: service and configuration

Implement the versioned D-Bus API and user service lifecycle. Add validated DPI,
lighting and polling-rate writes supported by each mode, plus readback where
possible. Test packet encoding/decoding against fixtures, timeout and reconnect
handling with a fake transport, and actual settings on the mouse. Separate manual
hardware checks from normal CI, which must work without a mouse attached.

### M3: Qt frontend and desktop behavior

Validate Qt/Rust integration, then implement device selection, capability-aware
controls and battery/charging display. Add session notifications with transition
tests. Verify KDE Plasma and GNOME behavior, including GUI closure and service
restart. Publish findings and scope for native UPower battery integration.

### M4: first usable Linux release

Package the frontend, user service, scoped access rules and a NixOS integration
module. Document installation, supported hardware/firmware, limitations and recovery
from disconnects. Verify installation and removal on a clean system, plus a manual
matrix covering USB receiver, BLE, suspend/resume and desktop notifications.

### Later

Button mapping, profiles, more mice and keyboards; Android transport/UI prototype.
Reassess API and capability modeling before adding keyboard-specific features.

## Open decisions

1. Exact Glorious model, hardware revision and firmware; USB and BLE descriptors.
2. Project license, to be chosen by the owner before distribution.
3. Qt/Rust bridge, after a build and packaging prototype.
4. Native battery-panel integration, after kernel/UPower inspection.

Items 1, 3 and 4 do not block the foundation PR. Scaffold implementation can proceed
without assigning a license, but publication requires the owner's license choice.

## References

- [Cargo workspaces](https://doc.rust-lang.org/stable/cargo/reference/workspaces.html)
- [Rust 2024 resolver](https://doc.rust-lang.org/stable/edition-guide/rust-2024/cargo-resolver.html)
- [Nix development environments](https://nix.dev/manual/nix/stable/command-ref/new-cli/nix3-develop)
- [HIDAPI and Linux backends](https://github.com/libusb/hidapi)
- [Desktop Notifications specification](https://specifications.freedesktop.org/notification/latest-single/)
- [UPower device interface](https://upower.freedesktop.org/docs/Device.html)
- [Qt for Android](https://doc.qt.io/qt-6/android.html)
