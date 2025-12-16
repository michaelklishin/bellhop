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
use clap::ArgMatches;

use crate::common::Project;
use crate::errors::BellhopError;
use crate::{aptly, cli};

pub fn add(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    let package_file_path = cli_args
        .get_one::<String>("package_file_path")
        .ok_or_else(|| BellhopError::MissingArgument {
            argument: "package_file_path".to_string(),
        })?;

    let target_releases = cli::distributions(cli_args, project)?;

    aptly::add_package(cli_args, package_file_path, project, &target_releases)
}

pub fn remove(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    let target_releases = cli::distributions(cli_args, project)?;

    if let Some(version) = cli_args.get_one::<String>("version") {
        aptly::remove_package(cli_args, version, project, &target_releases)
    } else if let Some(package_file_path) = cli_args.get_one::<String>("package_file_path") {
        aptly::remove_package_from_archive(cli_args, package_file_path, project, &target_releases)
    } else {
        Err(BellhopError::MissingArgument {
            argument: "version or package_file_path".to_string(),
        })
    }
}

pub fn publish(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    let target_releases = cli::distributions(cli_args, project)?;

    aptly::publish(project, &target_releases)
}

pub fn list_snapshots(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    aptly::list_snapshots(project, &target_releases, &suffix)
}

pub fn take_snapshots(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    aptly::take_snapshot(project, &target_releases, &suffix)
}

pub fn delete_snapshots(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    aptly::delete_snapshots(project, &target_releases, &suffix)
}
