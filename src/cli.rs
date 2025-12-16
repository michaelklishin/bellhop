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

use crate::common::Project;
use crate::deb::DistributionAlias;
use crate::errors::BellhopError;
use chrono::Local;
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};

pub fn parser() -> Command {
    Command::new("bellhop")
        .version(clap::crate_version!())
        .about("Puts input .deb and .rpm packages into the right places")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(rabbitmq_group())
        .subcommand(erlang_group())
}

pub fn distributions(
    cli_args: &ArgMatches,
    project: Project,
) -> Result<Vec<DistributionAlias>, BellhopError> {
    if cli_args.get_flag("all") {
        match project {
            Project::Erlang => Ok(DistributionAlias::erlang_supported().to_vec()),
            Project::RabbitMQ => Ok(DistributionAlias::all().to_vec()),
        }
    } else {
        cli_args
            .get_many::<String>("distributions")
            .ok_or_else(|| BellhopError::MissingArgument {
                argument: "distributions".to_string(),
            })?
            .map(|s| {
                s.as_str()
                    .parse::<DistributionAlias>()
                    .map_err(|_| BellhopError::InvalidDistribution { alias: s.clone() })
            })
            .collect()
    }
}

pub fn suffix(cli_args: &ArgMatches) -> String {
    let now = Local::now();
    let default = now.format("%d-%b-%y").to_string();

    cli_args
        .get_one::<String>("suffix")
        .cloned()
        .unwrap_or(default)
}

fn deb_group() -> Command {
    Command::new("deb")
        .about("Manage .deb packages")
        .arg_required_else_help(true)
        .subcommands(package_operation_subcommands())
}

fn rpm_group() -> Command {
    Command::new("rpm")
        .about("Manage .rpm packages")
        .arg_required_else_help(true)
        .subcommands(package_operation_subcommands())
}

fn rabbitmq_group() -> Command {
    Command::new("rabbitmq")
        .about("Manage RabbitMQ packages")
        .arg_required_else_help(true)
        .subcommands([deb_group(), rpm_group(), snapshot_group()])
}

fn erlang_group() -> Command {
    Command::new("erlang")
        .about("Manage Erlang packages")
        .arg_required_else_help(true)
        .subcommands([deb_group(), rpm_group(), snapshot_group()])
}

fn snapshot_group() -> Command {
    Command::new("snapshot")
        .about("Manage package repository snapshots")
        .arg_required_else_help(true)
        .subcommands(snapshot_subcommands())
}

fn add_distribution_args(cmd: Command, include_suffix: bool) -> Command {
    let mut cmd = cmd
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .conflicts_with("distributions")
                .help("Add the package to all distributions"),
        )
        .arg(
            Arg::new("distributions")
                .short('d')
                .long("distributions")
                .value_name("DISTRIBUTIONS")
                .conflicts_with("all")
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .help("A comma-separated list of distributions to add the package to"),
        )
        .group(
            ArgGroup::new("distribution")
                .args(["all", "distributions"])
                .required(true)
                .multiple(false),
        );

    if include_suffix {
        cmd = cmd.arg(
            Arg::new("suffix")
                .long("suffix")
                .value_name("NAME")
                .help("Snapshot suffix name, e.g. a date in the %d-%b-%y format, such as 04-Aug-25")
                .required(false),
        );
    }

    cmd
}

fn snapshot_subcommands() -> [Command; 3] {
    let list_cmd = add_distribution_args(Command::new("list").about("List snapshots"), true);
    let create_cmd = add_distribution_args(Command::new("take").about("Take a snapshot"), true);
    let delete_cmd = add_distribution_args(
        Command::new("delete")
            .about("Delete a snapshot")
            .visible_alias("remove"),
        true,
    );

    [list_cmd, create_cmd, delete_cmd]
}

fn package_operation_subcommands() -> [Command; 3] {
    let add_cmd = add_distribution_args(
        Command::new("add")
            .about("Add a package to one or multiple distributions")
            .arg(
                Arg::new("package_file_path")
                    .short('p')
                    .long("package-file-path")
                    .value_name("PATH")
                    .help("Binary package file path")
                    .required(true),
            ),
        true,
    );

    let remove_cmd = add_distribution_args(
        Command::new("remove")
            .about("Remove a .deb package from one or multiple distributions")
            .arg(
                Arg::new("version")
                    .short('v')
                    .long("version")
                    .value_name("VERSION")
                    .conflicts_with("package_file_path")
                    .help("Version of the package to remove"),
            )
            .arg(
                Arg::new("package_file_path")
                    .short('p')
                    .long("package-file-path")
                    .value_name("PATH")
                    .conflicts_with("version")
                    .help("Package file path (.deb, .zip, .tar.gz)"),
            )
            .group(
                ArgGroup::new("input")
                    .args(["version", "package_file_path"])
                    .required(true)
                    .multiple(false),
            ),
        true,
    );

    let publish_cmd = add_distribution_args(
        Command::new("publish").about(
            "Regenerates all repositories from recent snapshots (created by the 'add' command)",
        ),
        false,
    );

    [add_cmd, remove_cmd, publish_cmd]
}
