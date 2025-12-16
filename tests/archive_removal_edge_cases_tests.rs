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

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use tar::Builder;
use tempfile::TempDir;
use test_helpers::*;

fn add_packages_to_repo(
    ctx: &AptlyTestContext,
    repo_name: &str,
    packages: &[&str],
) -> Result<(), Box<dyn Error>> {
    for package in packages {
        let package_path = test_package_path(package);
        Command::new("aptly")
            .arg(ctx.config_arg())
            .arg("repo")
            .arg("add")
            .arg(repo_name)
            .arg(package_path.to_str().unwrap())
            .output()?;
    }
    Ok(())
}

fn create_tar_archive_with_debs(debs: &[&str]) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("packages.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            builder.append_path_with_name(&deb_path, deb)?;
        }
    }

    builder.finish()?;

    Ok((archive_path, temp_dir))
}

#[test]
fn test_remove_from_uncompressed_tar_archive() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.7-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should exist before removal"
    );

    let (archive_path, _temp_dir) =
        create_tar_archive_with_debs(&["rabbitmq-server_4.1.7-1_all.deb"])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should be removed from repository"
    );

    Ok(())
}

#[test]
fn test_remove_archive_with_same_version_different_packages() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(
        &ctx,
        repo_name,
        &[
            "rabbitmq-server_4.1.3-1_all.deb",
            "rabbitmq-server_4.1.4-1_all.deb",
            "rabbitmq-server_4.1.5-1_all.deb",
        ],
    )?;

    let (archive_path, _temp_dir) = create_tar_archive_with_debs(&[
        "rabbitmq-server_4.1.3-1_all.deb",
        "rabbitmq-server_4.1.4-1_all.deb",
        "rabbitmq-server_4.1.5-1_all.deb",
    ])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "First package should be removed"
    );
    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.4-1)")?,
        "Second package should be removed"
    );
    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.5-1)")?,
        "Third package should be removed"
    );

    Ok(())
}
