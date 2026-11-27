// SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::fmt::Debug;

use amanita::protocol::glorious_v3::commands::{BatteryInfo, DpiStage};
use amanita::protocol::glorious_v3::protocol::{
    GloriousReadCommand, GloriousV3, GloriousWriteCommand,
};
use amanita::transport::hid::{HidBackend, HidEndpoint};

fn find_endpoint(backend: &mut HidBackend, path: &str) -> Result<HidEndpoint, Box<dyn Error>> {
    let endpoints = backend.enumerate()?;

    endpoints
        .into_iter()
        .find(|endpoint| endpoint.path.to_str().ok() == Some(path))
        .ok_or_else(|| format!("HID path {path} not found").into())
}

fn get_cmd<T: GloriousReadCommand<Request = ()> + Debug>(
    protocol: &mut GloriousV3,
    arguments: &[String],
) -> Result<(), Box<dyn Error>> {
    if !arguments.is_empty() {
        return Err("expected no arguments for given command".into());
    }

    let response = protocol.read::<T>((), None)?;
    println!("{response:?}");
    Ok(())
}

fn set_cmd<T: GloriousWriteCommand>(
    protocol: &mut GloriousV3,
    args: &[String],
) -> Result<(), Box<dyn Error>> {
    let command = T::build(args)?;
    protocol.write(command, None)?;
    println!("Command succeeded");
    Ok(())
}

type Handler = fn(&mut GloriousV3, &[String]) -> Result<(), Box<dyn Error>>;

fn find_command_handler(operation: &str, command: &str) -> Result<Handler, Box<dyn Error>> {
    let commands: &[(&str, &str, Handler)] = &[
        ("get", "dpi_stage", get_cmd::<DpiStage>),
        ("get", "battery_info", get_cmd::<BatteryInfo>),
        ("set", "dpi_stage", set_cmd::<DpiStage>),
    ];

    let Some(handler) = commands
        .iter()
        .find_map(|&(op, name, handler)| (op == operation && name == command).then_some(handler))
    else {
        eprintln!("Available commands");
        for &(op, name, _) in commands {
            eprintln!("  {op} {name}");
        }
        return Err(format!("unknown command {operation} {command}").into());
    };

    Ok(handler)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let dev_path = args.nth(1).ok_or("expected a HID path")?;
    let operation = args.next().ok_or("expected a set/get operation")?;
    let command = args.next().ok_or("expected a command")?;
    let command_args: Vec<String> = args.collect();

    let command_handler = find_command_handler(&operation, &command)?;

    let mut backend = HidBackend::new()?;
    let endpoint = find_endpoint(&mut backend, &dev_path)?;
    let connection = backend.open(&endpoint)?;
    let mut protocol = GloriousV3::new(connection);

    command_handler(&mut protocol, command_args.as_slice())?;

    Ok(())
}
