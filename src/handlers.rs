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
use log::info;
use reqwest::blocking::Client;
use tempfile::TempDir;

use std::path::Path;

use crate::common::Project;
use crate::errors::BellhopError;
use crate::gh::{self, downloads, releases};
use crate::{aptly, cli, watcher};

pub fn add(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let package_file_path = cli_args
        .get_one::<String>("package_file_path")
        .ok_or_else(|| BellhopError::MissingArgument {
            argument: "package_file_path".to_string(),
        })?;

    let target_releases = cli::distributions(cli_args, project)?;

    aptly::add_package(cli_args, package_file_path, project, &target_releases)
}

pub fn remove(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

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
    aptly::check_aptly_available()?;

    let target_releases = cli::distributions(cli_args, project)?;

    aptly::publish(project, &target_releases)
}

pub fn list_snapshots(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    aptly::list_snapshots(project, &target_releases, &suffix)
}

pub fn take_snapshots(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    aptly::take_snapshot(project, &target_releases, &suffix)
}

pub fn delete_snapshots(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    aptly::delete_snapshots(project, &target_releases, &suffix)
}

pub fn import_from_github(cli_args: &ArgMatches, project: Project) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let url = cli_args
        .get_one::<String>("github_release_url")
        .ok_or_else(|| BellhopError::MissingArgument {
            argument: "github_release_url".to_string(),
        })?;

    let default_pattern = match project {
        Project::CliTools => "*amd64*.deb",
        Project::RabbitMQ | Project::Erlang => "*.deb",
    };
    let pattern = cli_args
        .get_one::<String>("pattern")
        .map(|s| s.as_str())
        .unwrap_or(default_pattern);

    let target_releases = cli::distributions(cli_args, project)?;
    let suffix = cli::suffix(cli_args);

    let release = gh::parse_release_url(url)?;
    info!(
        "Fetching release assets for {}/{} tag {}",
        release.owner, release.repo, release.tag
    );

    let client = Client::new();
    let assets = releases::fetch_release_assets(&client, &release)?;
    let filtered = releases::filter_assets(assets, pattern);

    if filtered.is_empty() {
        return Err(BellhopError::NoAssetsInRelease {
            pattern: pattern.to_string(),
        });
    }

    info!(
        "Found {} matching assets (pattern: '{pattern}')",
        filtered.len()
    );

    let temp_dir = TempDir::new()?;
    let downloaded = downloads::download_assets(&client, &filtered, temp_dir.path())?;

    for deb_path in &downloaded {
        aptly::add_single_package_no_snapshot(&project, deb_path, &target_releases)?;
    }
    aptly::update_snapshots_for_releases(&project, &target_releases, &suffix)?;

    info!(
        "Imported {} packages into {} distributions",
        downloaded.len(),
        target_releases.len()
    );
    Ok(())
}

pub fn setup_repositories() -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let existing = aptly::list_repos()?;
    let expected = aptly::expected_repos();

    let mut created = 0;
    for (project, repo) in &expected {
        if existing.contains(repo) {
            info!("Repository '{repo}' ({project}) already exists, skipping");
        } else {
            aptly::create_repo(repo)?;
            created += 1;
        }
    }

    info!(
        "Done: {created} repositories created, {} already existed",
        expected.len() - created
    );
    Ok(())
}

pub fn watch(cli_args: &ArgMatches) -> Result<(), BellhopError> {
    aptly::check_aptly_available()?;

    let root = cli_args
        .get_one::<String>("root")
        .ok_or_else(|| BellhopError::MissingArgument {
            argument: "root".to_string(),
        })?;

    let target_releases = cli::distributions_for_all_projects(cli_args)?;

    watcher::watch_directory(Path::new(root), &target_releases, None)
}
