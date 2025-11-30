// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(dead_code)]

mod aptly;
mod archive;
mod cli;
mod common;
mod deb;
mod errors;
mod handlers;

use common::Project;
use errors::{BellhopError, ExitCode, map_error_to_exit_code};
use std::process;

fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log::LevelFilter::Info)
        .level_for("bellhop", log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn main() {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize logging: {e}");
    }

    let parser = cli::parser();
    let cli_args = parser.get_matches();

    let exit_code = match run(&cli_args) {
        Ok(_) => ExitCode::Ok,
        Err(err) => {
            eprintln!("Error: {err}");
            map_error_to_exit_code(&err)
        }
    };

    process::exit(exit_code.into());
}

fn run(cli_args: &clap::ArgMatches) -> Result<(), BellhopError> {
    if let Some((first_level, first_level_args)) = cli_args.subcommand()
        && let Some((second_level, second_level_args)) = first_level_args.subcommand()
        && let Some((third_level, third_level_args)) = second_level_args.subcommand()
    {
        return dispatch_command(first_level, second_level, third_level, third_level_args);
    }
    Ok(())
}

fn dispatch_command(
    first_level: &str,
    second_level: &str,
    third_level: &str,
    third_level_args: &clap::ArgMatches,
) -> Result<(), BellhopError> {
    let project = match first_level {
        "rabbitmq" => Project::RabbitMQ,
        "erlang" => Project::Erlang,
        _ => {
            return Err(BellhopError::UnknownCommand {
                first: first_level.to_string(),
                second: second_level.to_string(),
                third: third_level.to_string(),
            });
        }
    };

    match (second_level, third_level) {
        ("deb", "add") => handlers::add(third_level_args, project),
        ("deb", "remove") => handlers::remove(third_level_args, project),
        ("deb", "publish") => handlers::publish(third_level_args, project),
        ("snapshot", "take") => handlers::take_snapshots(third_level_args, project),
        ("snapshot", "delete") => handlers::delete_snapshots(third_level_args, project),
        ("snapshot", "list") => handlers::list_snapshots(third_level_args, project),
        _ => Err(BellhopError::UnknownCommand {
            first: first_level.to_string(),
            second: second_level.to_string(),
            third: third_level.to_string(),
        }),
    }
}
