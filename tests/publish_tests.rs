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
use chrono::Local;
use std::error::Error;
use std::process::Command;
use test_helpers::*;

#[test]
fn test_publish_single_distribution() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;
    ctx.create_initial_publish("rabbitmq-server", "debian", "bookworm")?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["rabbitmq", "deb", "publish", "-d", "bookworm"]);
    cmd.assert().success();

    let date = Local::now().format("%d-%b-%y").to_string();
    let expected_snapshot = format!("snap-rabbitmq-server-bookworm-{}", date);
    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-server",
            "debian",
            "bookworm",
            &expected_snapshot
        )?,
        "Published repository should use the new snapshot"
    );

    Ok(())
}

#[test]
fn test_publish_multiple_distributions() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;
    ctx.create_initial_publish("rabbitmq-server", "debian", "bookworm")?;
    ctx.create_initial_publish("rabbitmq-server", "ubuntu", "jammy")?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm,jammy",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["rabbitmq", "deb", "publish", "-d", "bookworm,jammy"]);
    cmd.assert().success();

    let date = Local::now().format("%d-%b-%y").to_string();
    let expected_bookworm = format!("snap-rabbitmq-server-bookworm-{}", date);
    let expected_jammy = format!("snap-rabbitmq-server-jammy-{}", date);

    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-server",
            "debian",
            "bookworm",
            &expected_bookworm
        )?,
        "Bookworm should use new snapshot"
    );
    assert!(
        ctx.published_snapshot_is_active("rabbitmq-server", "ubuntu", "jammy", &expected_jammy)?,
        "Jammy should use new snapshot"
    );

    Ok(())
}

#[test]
fn test_full_workflow_add_publish_upgrade() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;
    ctx.create_initial_publish("rabbitmq-server", "debian", "bookworm")?;

    let package_path_v1 = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        package_path_v1.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["rabbitmq", "deb", "publish", "-d", "bookworm"]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Version 4.1.3 should be in repository"
    );

    // Add second package (simulating an upgrade on a different day by using a custom suffix)
    let package_path_v2 = test_package_path("rabbitmq-server_4.1.4-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        package_path_v2.to_str().unwrap(),
        "-d",
        "bookworm",
        "--suffix",
        "v2",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Old version should still exist"
    );
    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.4-1)")?,
        "New version should be added"
    );

    // Verify the first snapshot is still published (we didn't publish the second one)
    let date = Local::now().format("%d-%b-%y").to_string();
    let expected_snapshot = format!("snap-rabbitmq-server-bookworm-{}", date);
    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-server",
            "debian",
            "bookworm",
            &expected_snapshot
        )?,
        "Should still use the first snapshot (second was not published)"
    );

    // Verify the second snapshot exists but is not published
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bookworm-v2")?,
        "Second snapshot with custom suffix should exist"
    );

    Ok(())
}

#[test]
fn test_erlang_publish_workflow() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-erlang-bookworm";
    ctx.create_repo(repo_name)?;
    ctx.create_initial_publish("rabbitmq-erlang", "debian", "bookworm")?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "erlang",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["erlang", "deb", "publish", "-d", "bookworm"]);
    cmd.assert().success();

    let date = Local::now().format("%d-%b-%y").to_string();
    let expected_snapshot = format!("snap-rabbitmq-erlang-bookworm-{}", date);
    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-erlang",
            "debian",
            "bookworm",
            &expected_snapshot
        )?,
        "Erlang project should have separate published snapshot"
    );

    Ok(())
}

#[test]
fn test_publish_all_distributions() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    // Create repos and publishes for all supported distributions
    ctx.create_repo("repo-rabbitmq-server-noble")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;
    ctx.create_repo("repo-rabbitmq-server-focal")?;
    ctx.create_repo("repo-rabbitmq-server-trixie")?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-bullseye")?;
    ctx.create_initial_publish("rabbitmq-server", "ubuntu", "noble")?;
    ctx.create_initial_publish("rabbitmq-server", "ubuntu", "jammy")?;
    ctx.create_initial_publish("rabbitmq-server", "ubuntu", "focal")?;
    ctx.create_initial_publish("rabbitmq-server", "debian", "trixie")?;
    ctx.create_initial_publish("rabbitmq-server", "debian", "bookworm")?;
    ctx.create_initial_publish("rabbitmq-server", "debian", "bullseye")?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-a",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["rabbitmq", "deb", "publish", "-a"]);
    cmd.assert().success();

    let date = Local::now().format("%d-%b-%y").to_string();

    // Verify at least a few distributions were published
    let expected_noble = format!("snap-rabbitmq-server-noble-{}", date);
    let expected_bookworm = format!("snap-rabbitmq-server-bookworm-{}", date);
    let expected_jammy = format!("snap-rabbitmq-server-jammy-{}", date);

    assert!(
        ctx.published_snapshot_is_active("rabbitmq-server", "ubuntu", "noble", &expected_noble)?,
        "Noble should use new snapshot"
    );
    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-server",
            "debian",
            "bookworm",
            &expected_bookworm
        )?,
        "Bookworm should use new snapshot"
    );
    assert!(
        ctx.published_snapshot_is_active("rabbitmq-server", "ubuntu", "jammy", &expected_jammy)?,
        "Jammy should use new snapshot"
    );

    Ok(())
}

#[test]
fn test_publish_new_distribution_without_initial_publish() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["rabbitmq", "deb", "publish", "-d", "bookworm"]);
    cmd.assert().success();

    let date = Local::now().format("%d-%b-%y").to_string();
    let expected_snapshot = format!("snap-rabbitmq-server-bookworm-{date}");

    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-server",
            "debian",
            "bookworm",
            &expected_snapshot
        )?,
        "Bookworm should be published even without initial publish setup"
    );

    Ok(())
}

#[test]
fn test_erlang_publish_new_distribution() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-erlang-trixie";
    ctx.create_repo(repo_name)?;

    let package_path = test_package_path("erlang-base_27.3.4.6-1_amd64.deb");
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "erlang",
        "deb",
        "add",
        "-p",
        package_path.to_str().unwrap(),
        "-d",
        "trixie",
    ]);
    cmd.assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["erlang", "deb", "publish", "-d", "trixie"]);
    cmd.assert().success();

    let date = Local::now().format("%d-%b-%y").to_string();
    let expected_snapshot = format!("snap-rabbitmq-erlang-trixie-{date}");

    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-erlang",
            "debian",
            "trixie",
            &expected_snapshot
        )?,
        "Erlang Trixie should be published without initial publish setup"
    );

    Ok(())
}
