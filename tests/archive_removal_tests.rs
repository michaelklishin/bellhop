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
use std::process::Command;
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

#[test]
fn test_remove_nonexistent_archive_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-p",
        "/nonexistent/archive.zip",
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("Package file does not exist"));

    Ok(())
}

#[test]
fn test_remove_single_deb_file() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.3-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package should exist before removal"
    );

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package should be removed from repository"
    );

    Ok(())
}

#[test]
fn test_remove_preserves_existing_version_flag() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.3-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package should exist before removal"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq", "deb", "remove", "-v", "4.1.3-1", "-d", "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package should be removed from repository"
    );

    Ok(())
}

#[test]
fn test_remove_version_and_path_conflict() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-v",
        "4.1.3-1",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("cannot be used with"));

    Ok(())
}
