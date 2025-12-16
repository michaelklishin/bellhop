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

fn fixture_archive_available(filename: &str) -> bool {
    test_fixture_path(filename).exists()
}

#[test]
fn test_remove_package_from_real_zip_archive() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.7-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should exist before removal"
    );

    let archive_path = test_fixture_path("archives/rabbitmq-4.1.7.zip");
    if !fixture_archive_available("archives/rabbitmq-4.1.7.zip") {
        eprintln!("Skipping test: fixture archive not available");
        return Ok(());
    }

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
fn test_remove_package_from_real_tar_gz_archive() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.7-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should exist before removal"
    );

    let archive_path = test_fixture_path("archives/rabbitmq-4.1.7.tar.gz");
    if !fixture_archive_available("archives/rabbitmq-4.1.7.tar.gz") {
        eprintln!("Skipping test: fixture archive not available");
        return Ok(());
    }

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
fn test_remove_multiple_packages_from_real_zip() -> Result<(), Box<dyn Error>> {
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

    assert!(ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?);
    assert!(ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.4-1)")?);
    assert!(ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.5-1)")?);

    let archive_path = test_fixture_path("archives/rabbitmq-multi.zip");
    if !fixture_archive_available("archives/rabbitmq-multi.zip") {
        eprintln!("Skipping test: fixture archive not available");
        return Ok(());
    }

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

#[test]
fn test_remove_multiple_packages_from_real_tar_gz() -> Result<(), Box<dyn Error>> {
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

    let archive_path = test_fixture_path("archives/rabbitmq-multi.tar.gz");
    if !fixture_archive_available("archives/rabbitmq-multi.tar.gz") {
        eprintln!("Skipping test: fixture archive not available");
        return Ok(());
    }

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

#[test]
fn test_remove_real_deb_file_directly() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.7-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should exist before removal"
    );

    let package_path = test_package_path("rabbitmq-server_4.1.7-1_all.deb");

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
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should be removed from repository"
    );

    Ok(())
}

#[test]
fn test_remove_from_multiple_distributions_with_real_archive() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;

    add_packages_to_repo(
        &ctx,
        "repo-rabbitmq-server-bookworm",
        &["rabbitmq-server_4.1.7-1_all.deb"],
    )?;
    add_packages_to_repo(
        &ctx,
        "repo-rabbitmq-server-jammy",
        &["rabbitmq-server_4.1.7-1_all.deb"],
    )?;

    let archive_path = test_fixture_path("archives/rabbitmq-4.1.7.zip");
    if !fixture_archive_available("archives/rabbitmq-4.1.7.zip") {
        eprintln!("Skipping test: fixture archive not available");
        return Ok(());
    }

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm,jammy",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(
            "repo-rabbitmq-server-bookworm",
            "rabbitmq-server (= 4.1.7-1)"
        )?,
        "Package should be removed from bookworm"
    );
    assert!(
        !ctx.package_exists("repo-rabbitmq-server-jammy", "rabbitmq-server (= 4.1.7-1)")?,
        "Package should be removed from jammy"
    );

    Ok(())
}

#[test]
fn test_remove_erlang_package_from_zip_for_trixie() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-trixie";
    ctx.create_repo(repo_name)?;

    add_packages_to_repo(&ctx, repo_name, &["rabbitmq-server_4.1.7-1_all.deb"])?;

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should exist before removal"
    );

    let archive_path = test_fixture_path("archives/rabbitmq-4.1.7.zip");
    if !fixture_archive_available("archives/rabbitmq-4.1.7.zip") {
        eprintln!("Skipping test: fixture archive not available");
        return Ok(());
    }

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "trixie",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package should be removed from repository"
    );

    Ok(())
}
