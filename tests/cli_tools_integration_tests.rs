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

#[test]
fn test_cli_tools_has_no_rpm_subcommand() -> Result<(), Box<dyn Error>> {
    run_bellhop_fails(["cli-tools", "rpm"]);
    Ok(())
}

#[test]
fn test_cli_tools_deb_add_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["cli-tools", "deb", "add", "--help"]).stdout(output_includes(
        "Add a package to one or multiple distributions",
    ));
    Ok(())
}

#[test]
fn test_cli_tools_deb_remove_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["cli-tools", "deb", "remove", "--help"]).stdout(output_includes(
        "Remove a .deb package from one or multiple distributions",
    ));
    Ok(())
}

#[test]
fn test_cli_tools_snapshot_take_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["cli-tools", "snapshot", "take", "--help"])
        .stdout(output_includes("Take a snapshot"));
    Ok(())
}

#[test]
fn test_cli_tools_add_package_to_single_distribution() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-cli-bookworm";

    ctx.create_repo(repo_name)?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "cli-tools",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let packages = ctx.list_packages(repo_name)?;
    assert!(
        !packages.is_empty(),
        "Package should exist in cli-tools repository"
    );

    let snapshots = ctx.list_snapshots("snap-rabbitmq-cli-bookworm")?;
    assert!(
        !snapshots.is_empty(),
        "Snapshot should be created after adding package"
    );

    Ok(())
}

#[test]
fn test_cli_tools_remove_package_by_version() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-cli-bookworm";

    ctx.create_repo(repo_name)?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("add")
        .arg(repo_name)
        .arg(package_path.to_str().unwrap())
        .output()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "cli-tools",
        "deb",
        "remove",
        "-v",
        "4.1.3-1",
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let packages = ctx.list_packages(repo_name)?;
    assert!(
        packages.is_empty(),
        "Package should be removed from repository"
    );

    Ok(())
}

#[test]
fn test_cli_tools_snapshot_take() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-cli-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "cli-tools",
        "snapshot",
        "take",
        "-d",
        "bookworm",
        "--suffix",
        "cli-01",
    ]);
    cmd.assert().success();

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-cli-bookworm-cli-01")?,
        "CLI tools snapshot should use rabbitmq-cli prefix"
    );

    Ok(())
}

#[test]
fn test_cli_tools_all_flag_uses_all_distributions() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-cli-noble")?;
    ctx.create_repo("repo-rabbitmq-cli-jammy")?;
    ctx.create_repo("repo-rabbitmq-cli-focal")?;
    ctx.create_repo("repo-rabbitmq-cli-trixie")?;
    ctx.create_repo("repo-rabbitmq-cli-bookworm")?;
    ctx.create_repo("repo-rabbitmq-cli-bullseye")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["cli-tools", "snapshot", "take", "--all", "--suffix", "test"]);
    cmd.assert().success();

    for dist in ["noble", "jammy", "focal", "trixie", "bookworm", "bullseye"] {
        assert!(
            ctx.snapshot_exists(&format!("snap-rabbitmq-cli-{dist}-test"))?,
            "CLI tools should support {dist}"
        );
    }

    Ok(())
}
