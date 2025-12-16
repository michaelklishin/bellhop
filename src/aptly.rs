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
use crate::archive::{self, PackageSource};
use crate::deb::DistributionAlias;
use crate::errors::BellhopError;
use crate::{cli, common::Project};
use chrono::Local;
use clap::ArgMatches;
use log::{debug, info};
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ALL_ARCHITECTURES_ARG: &str = "-architectures=amd64,arm64,armel,armhf,i386";
const GPG_KEY_ID_ARG: &str = "-gpg-key=0A9AF2115F4687BD29803A206B73A36E6026DFCA";

static APTLY_AVAILABLE: OnceLock<bool> = OnceLock::new();

pub fn check_aptly_available() -> Result<(), BellhopError> {
    let available = APTLY_AVAILABLE.get_or_init(|| {
        Command::new("aptly")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    });

    if *available {
        Ok(())
    } else {
        Err(BellhopError::AptlyNotFound)
    }
}

fn aptly_command() -> Command {
    let mut cmd = Command::new("aptly");
    if let Ok(config_path) = env::var("APTLY_CONFIG") {
        cmd.arg(format!("-config={config_path}"));
    }
    cmd
}

fn check_aptly_output(output: Output, command: impl Into<String>) -> Result<Output, BellhopError> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(BellhopError::AptlyNonZeroExit {
            command: command.into(),
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub fn add_package(
    cli_args: &ArgMatches,
    package_file_path: &str,
    project: Project,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    let path = PathBuf::from(package_file_path);
    if !path.exists() {
        return Err(BellhopError::PackageFileNotFound { path });
    }

    info!("Processing package file: {}", path.display());
    let package_source = archive::process_package_file(&path)?;

    let suffix = cli::suffix(cli_args);

    match package_source {
        PackageSource::SingleDeb(deb_path) => {
            info!("Adding single .deb package");
            add_single_package(cli_args, &deb_path, project, target_releases)?;
        }
        PackageSource::Archive {
            deb_files,
            _temp_dir,
        } => {
            info!("Adding {} packages from archive", deb_files.len());
            for deb_path in &deb_files {
                debug!("Processing: {}", deb_path.display());
                add_single_package_no_snapshot(&project, deb_path, target_releases)?;
            }
            update_snapshots_for_releases(&project, target_releases, &suffix)?;
        }
    }

    Ok(())
}

fn update_snapshots_for_releases(
    project: &Project,
    target_releases: &[DistributionAlias],
    suffix: &str,
) -> Result<(), BellhopError> {
    for rel in target_releases {
        let repo_name = repo_name(project, rel);
        run_snapshot_drop(project, rel, suffix)?;
        run_snapshot_create(project, &repo_name, rel, suffix)?;
    }
    Ok(())
}

fn add_single_package(
    cli_args: &ArgMatches,
    deb_path: &Path,
    project: Project,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    let suffix = cli::suffix(cli_args);

    for rel in target_releases {
        let repo_name = repo_name(&project, rel);
        run_repo_add(&project, deb_path, &repo_name, rel)?;
    }
    update_snapshots_for_releases(&project, target_releases, &suffix)
}

fn add_single_package_no_snapshot(
    project: &Project,
    deb_path: &Path,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    for rel in target_releases {
        let repo_name = repo_name(project, rel);
        run_repo_add(project, deb_path, &repo_name, rel)?;
    }
    Ok(())
}

pub fn remove_package(
    cli_args: &ArgMatches,
    version: &str,
    project: Project,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    let suffix = cli::suffix(cli_args);

    for rel in target_releases {
        let repo_name = repo_name(&project, rel);
        run_repo_remove(&project, version, &repo_name)?;
    }
    update_snapshots_for_releases(&project, target_releases, &suffix)
}

pub fn remove_package_from_archive(
    cli_args: &ArgMatches,
    package_file_path: &str,
    project: Project,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    let path = PathBuf::from(package_file_path);
    if !path.exists() {
        return Err(BellhopError::PackageFileNotFound { path });
    }

    info!("Processing package file: {}", path.display());
    let package_source = archive::process_package_file(&path)?;

    let suffix = cli::suffix(cli_args);

    match package_source {
        PackageSource::SingleDeb(deb_path) => {
            info!("Removing single .deb package");
            let version = archive::extract_version_from_deb(&deb_path)?;
            remove_single_package(cli_args, &version, project, target_releases)?;
        }
        PackageSource::Archive {
            deb_files,
            _temp_dir,
        } => {
            info!("Removing {} packages from archive", deb_files.len());
            let versions = archive::extract_versions_from_debs(&deb_files)?;
            let unique_versions: HashSet<String> = versions.into_iter().collect();

            info!(
                "Found {} unique version(s) to remove",
                unique_versions.len()
            );
            for version in &unique_versions {
                debug!("Removing version: {version}");
                remove_single_package_no_snapshot(&project, version, target_releases)?;
            }
            update_snapshots_for_releases(&project, target_releases, &suffix)?;
        }
    }

    Ok(())
}

fn remove_single_package(
    cli_args: &ArgMatches,
    version: &str,
    project: Project,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    let suffix = cli::suffix(cli_args);

    for rel in target_releases {
        let repo_name = repo_name(&project, rel);
        run_repo_remove(&project, version, &repo_name)?;
        run_snapshot_drop(&project, rel, &suffix)?;
        run_snapshot_create(&project, &repo_name, rel, &suffix)?;
    }
    Ok(())
}

fn remove_single_package_no_snapshot(
    project: &Project,
    version: &str,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    for rel in target_releases {
        let repo_name = repo_name(project, rel);
        run_repo_remove(project, version, &repo_name)?;
    }
    Ok(())
}

pub fn publish(
    project: Project,
    target_releases: &[DistributionAlias],
) -> Result<(), BellhopError> {
    for rel in target_releases {
        run_snapshot_switch(&project, rel)?;
    }
    Ok(())
}

pub fn list_snapshots(
    project: Project,
    target_releases: &[DistributionAlias],
    suffix: &str,
) -> Result<(), BellhopError> {
    for rel in target_releases {
        run_snapshot_show(&project, rel, suffix)?;
    }
    Ok(())
}

pub fn take_snapshot(
    project: Project,
    target_releases: &[DistributionAlias],
    suffix: &str,
) -> Result<(), BellhopError> {
    for rel in target_releases {
        let repo_name = repo_name(&project, rel);
        run_snapshot_create(&project, &repo_name, rel, suffix)?;
    }
    Ok(())
}

pub fn delete_snapshots(
    project: Project,
    target_releases: &[DistributionAlias],
    suffix: &str,
) -> Result<(), BellhopError> {
    for rel in target_releases {
        run_snapshot_drop(&project, rel, suffix)?;
    }
    Ok(())
}

pub fn repo_name(project: &Project, rel: &DistributionAlias) -> String {
    match project {
        Project::RabbitMQ => {
            format!("repo-rabbitmq-server-{rel}")
        }
        Project::Erlang => {
            format!("repo-rabbitmq-erlang-{rel}")
        }
    }
}

fn snapshot_name(project: &Project, rel: &DistributionAlias) -> String {
    let date = Local::now().format("%d-%b-%y");
    let prefix = project_prefix(project);

    format!("snap-{}-{}-{}", prefix, rel.release_name(), date)
}

pub fn snapshot_name_with_suffix(
    project: &Project,
    rel: &DistributionAlias,
    suffix: &str,
) -> String {
    let prefix = project_prefix(project);

    format!("snap-{}-{}-{}", prefix, rel.release_name(), suffix)
}

pub fn rel_path_with_prefix(project: &Project, rel: &DistributionAlias) -> String {
    let prefix = project_prefix(project);
    format!("{}/{}/{}", prefix, rel.family_name(), rel.release_name())
}

pub fn project_prefix(project: &Project) -> &'static str {
    match project {
        Project::RabbitMQ => "rabbitmq-server",
        Project::Erlang => "rabbitmq-erlang",
    }
}

fn run_repo_add(
    project: &Project,
    package_file_path: &Path,
    repo_name: &str,
    rel: &DistributionAlias,
) -> Result<(), BellhopError> {
    let path_str = package_file_path.display();
    info!("Adding package {path_str} to repo '{repo_name}' for distribution '{rel}'");

    let output = aptly_command()
        .arg("repo")
        .arg("add")
        .args(matches!(project, Project::RabbitMQ).then_some(ALL_ARCHITECTURES_ARG))
        .arg(repo_name)
        .arg(package_file_path)
        .output()?;
    check_aptly_output(output, format!("aptly repo add {repo_name} {path_str}"))?;

    debug!("Package added successfully");
    Ok(())
}

fn run_repo_remove(project: &Project, version: &str, repo_name: &str) -> Result<(), BellhopError> {
    let query = match project {
        Project::RabbitMQ => format!("rabbitmq-server (= {version})"),
        Project::Erlang => format!("Name (~ ^erlang), Version (= {version})"),
    };

    info!("Removing packages matching query '{query}' from repo '{repo_name}'");

    let output = aptly_command()
        .arg("repo")
        .arg("remove")
        .arg(repo_name)
        .arg(&query)
        .output()?;

    check_aptly_output(output, format!("aptly repo remove {repo_name} {query}"))?;
    Ok(())
}

fn run_snapshot_show(
    project: &Project,
    rel: &DistributionAlias,
    suffix: &str,
) -> Result<(), BellhopError> {
    let snapshot_name = snapshot_name_with_suffix(project, rel, suffix);

    let output = aptly_command()
        .arg("snapshot")
        .arg("show")
        .arg("-with-packages")
        .arg(&snapshot_name)
        .output()?;

    let output = check_aptly_output(
        output,
        format!("aptly snapshot show -with-packages {snapshot_name}"),
    )?;

    print!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

fn run_snapshot_create(
    project: &Project,
    repo_name: &str,
    rel: &DistributionAlias,
    suffix: &str,
) -> Result<(), BellhopError> {
    let snapshot_name = snapshot_name_with_suffix(project, rel, suffix);
    info!("Creating snapshot '{snapshot_name}' from repo '{repo_name}'");

    let output = aptly_command()
        .arg("snapshot")
        .arg("create")
        .arg(&snapshot_name)
        .arg("from")
        .arg("repo")
        .arg(repo_name)
        .output()?;

    check_aptly_output(
        output,
        format!("aptly snapshot create {snapshot_name} from repo {repo_name}"),
    )?;

    info!("Snapshot created successfully: {snapshot_name}");
    Ok(())
}

fn run_snapshot_drop(
    project: &Project,
    rel: &DistributionAlias,
    suffix: &str,
) -> Result<(), BellhopError> {
    let snapshot_name = snapshot_name_with_suffix(project, rel, suffix);

    debug!("Dropping snapshot '{snapshot_name}'");

    // Drop is allowed to fail (snapshot may not exist)
    // Use -force to allow dropping published snapshots
    // Ignore all errors including IO errors
    let output = aptly_command()
        .arg("snapshot")
        .arg("drop")
        .arg("-force")
        .arg(&snapshot_name)
        .output();

    if let Ok(out) = output {
        if !out.status.success() {
            debug!(
                "Snapshot drop failed (this is okay): {}",
                String::from_utf8_lossy(&out.stderr)
            );
        } else {
            debug!("Snapshot dropped successfully");
        }
    }

    Ok(())
}

fn run_snapshot_switch(project: &Project, rel: &DistributionAlias) -> Result<(), BellhopError> {
    let snapshot_name = snapshot_name(project, rel);
    let rel_path = rel_path_with_prefix(project, rel);

    info!("Publishing snapshot '{snapshot_name}' to '{rel_path}'");

    if publication_exists(&rel_path, rel.release_name())? {
        let output = aptly_command()
            .arg("publish")
            .arg("switch")
            .arg(GPG_KEY_ID_ARG)
            .arg(rel.release_name())
            .arg(&rel_path)
            .arg(&snapshot_name)
            .output()?;

        check_aptly_output(
            output,
            format!(
                "aptly publish switch {} {} {} {}",
                GPG_KEY_ID_ARG,
                rel.release_name(),
                rel_path,
                snapshot_name
            ),
        )?;
    } else {
        debug!("Publication does not exist, using 'publish snapshot' instead of 'switch'");

        let output = aptly_command()
            .arg("publish")
            .arg("snapshot")
            .arg("-distribution")
            .arg(rel.release_name())
            .arg(GPG_KEY_ID_ARG)
            .arg(&snapshot_name)
            .arg(&rel_path)
            .output()?;

        check_aptly_output(
            output,
            format!(
                "aptly publish snapshot -distribution {} {} {} {}",
                rel.release_name(),
                GPG_KEY_ID_ARG,
                snapshot_name,
                rel_path
            ),
        )?;
    }

    Ok(())
}

fn publication_exists(prefix: &str, distribution: &str) -> Result<bool, BellhopError> {
    let output = aptly_command().arg("publish").arg("list").output()?;
    let output = check_aptly_output(output, "aptly publish list")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let search_pattern = format!("{prefix}/{distribution}");
    Ok(stdout.contains(&search_pattern))
}
