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
mod test_helpers;

use bellhop::common::Project;
use bellhop::deb::DistributionAlias;
use bellhop::watcher;
use std::env;
use std::error::Error;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use test_helpers::*;

#[test]
fn test_watch_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["watch", "--help"])
        .stdout(output_includes("Watch directories for .deb files"))
        .stdout(output_includes("--root"));
    Ok(())
}

#[test]
fn test_watch_requires_root() -> Result<(), Box<dyn Error>> {
    run_bellhop_fails(["watch", "--all"])
        .stderr(output_includes("required arguments were not provided"));
    Ok(())
}

#[test]
fn test_project_for_directory_rabbitmq_server() {
    assert_eq!(
        watcher::project_for_directory("rabbitmq-server"),
        Some(Project::RabbitMQ)
    );
}

#[test]
fn test_project_for_directory_rabbitmq_erlang() {
    assert_eq!(
        watcher::project_for_directory("rabbitmq-erlang"),
        Some(Project::Erlang)
    );
}

#[test]
fn test_project_for_directory_rabbitmq_cli() {
    assert_eq!(
        watcher::project_for_directory("rabbitmq-cli"),
        Some(Project::CliTools)
    );
}

#[test]
fn test_project_for_directory_unknown() {
    assert_eq!(watcher::project_for_directory("unknown"), None);
    assert_eq!(watcher::project_for_directory(""), None);
}

#[test]
fn test_releases_for_project_filters_erlang() {
    let all = DistributionAlias::all().to_vec();
    let erlang_dists = watcher::releases_for_project(&Project::Erlang, &all);
    assert_eq!(erlang_dists.len(), 4);
    assert!(!erlang_dists.contains(&&DistributionAlias::Focal));
    assert!(!erlang_dists.contains(&&DistributionAlias::Bullseye));
}

#[test]
fn test_releases_for_project_passes_all_for_rabbitmq() {
    let all = DistributionAlias::all().to_vec();
    let rabbitmq_dists = watcher::releases_for_project(&Project::RabbitMQ, &all);
    assert_eq!(rabbitmq_dists.len(), 6);
}

#[test]
fn test_releases_for_project_passes_all_for_cli_tools() {
    let all = DistributionAlias::all().to_vec();
    let cli_dists = watcher::releases_for_project(&Project::CliTools, &all);
    assert_eq!(cli_dists.len(), 6);
}

#[test]
fn test_watch_creates_subdirectories() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let watch_root = temp_dir.path().join("watch");
    fs::create_dir_all(&watch_root)?;

    let dists = vec![DistributionAlias::Bookworm];

    watcher::watch_directory(&watch_root, &dists, Some(0))?;

    assert!(watch_root.join("rabbitmq-server").exists());
    assert!(watch_root.join("rabbitmq-erlang").exists());
    assert!(watch_root.join("rabbitmq-cli").exists());

    Ok(())
}

#[test]
fn test_watch_imports_deb_on_create() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let watch_root = ctx.temp_dir.path().join("watch");
    fs::create_dir_all(&watch_root)?;

    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let dists = vec![DistributionAlias::Bookworm];

    let config_path = ctx.config_path.clone();
    let watch_root_clone = watch_root.clone();

    let handle = thread::spawn(move || {
        unsafe {
            env::set_var("APTLY_CONFIG", config_path.to_str().unwrap());
        }
        watcher::watch_directory(&watch_root_clone, &dists, Some(1))
    });

    thread::sleep(Duration::from_millis(500));

    let src = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let dest = watch_root
        .join("rabbitmq-server")
        .join("rabbitmq-server_4.1.3-1_all.deb");
    fs::copy(&src, &dest)?;

    let timeout = Duration::from_secs(10);
    let start = Instant::now();
    loop {
        if handle.is_finished() {
            break;
        }
        if start.elapsed() > timeout {
            panic!("Watcher thread did not finish within timeout");
        }
        thread::sleep(Duration::from_millis(100));
    }

    let result = handle.join().unwrap();
    assert!(result.is_ok(), "Watcher should succeed: {result:?}");

    assert!(ctx.package_exists(repo_name, "rabbitmq-server")?);

    Ok(())
}
