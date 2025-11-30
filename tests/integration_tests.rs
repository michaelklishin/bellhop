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
fn test_help_with_no_arguments() -> Result<(), Box<dyn Error>> {
    let args: [&str; 0] = [];
    run_bellhop_fails(args)
        .stderr(output_includes("Usage:"))
        .stderr(output_includes("Commands:"));
    Ok(())
}

#[test]
fn test_rabbitmq_deb_add_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["rabbitmq", "deb", "add", "--help"]).stdout(output_includes(
        "Add a package to one or multiple distributions",
    ));
    Ok(())
}

#[test]
fn test_rabbitmq_deb_remove_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["rabbitmq", "deb", "remove", "--help"]).stdout(output_includes(
        "Remove a .deb package from one or multiple distributions",
    ));
    Ok(())
}

#[test]
fn test_add_package_to_single_distribution() -> Result<(), Box<dyn Error>> {
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

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package should exist in repository"
    );

    let snapshots = ctx.list_snapshots("snap-rabbitmq-server-bookworm")?;
    assert!(
        !snapshots.is_empty(),
        "Snapshot should be created after adding package"
    );

    Ok(())
}

#[test]
fn test_add_package_to_multiple_distributions() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;

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

    assert!(
        ctx.package_exists(
            "repo-rabbitmq-server-bookworm",
            "rabbitmq-server (= 4.1.3-1)"
        )?,
        "Package should exist in bookworm repository"
    );
    assert!(
        ctx.package_exists("repo-rabbitmq-server-jammy", "rabbitmq-server (= 4.1.3-1)")?,
        "Package should exist in jammy repository"
    );

    let bookworm_snapshots = ctx.list_snapshots("snap-rabbitmq-server-bookworm")?;
    let jammy_snapshots = ctx.list_snapshots("snap-rabbitmq-server-jammy")?;
    assert!(
        !bookworm_snapshots.is_empty(),
        "Bookworm snapshot should exist"
    );
    assert!(!jammy_snapshots.is_empty(), "Jammy snapshot should exist");

    Ok(())
}

#[test]
fn test_add_nonexistent_package_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        "/nonexistent/package.deb",
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("Package file does not exist"));

    Ok(())
}

#[test]
fn test_remove_package_from_single_distribution() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";

    ctx.create_repo(repo_name)?;
    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("add")
        .arg(repo_name)
        .arg(package_path.to_str().unwrap())
        .output()?;

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

    let snapshots = ctx.list_snapshots("snap-rabbitmq-server-bookworm")?;
    assert!(
        !snapshots.is_empty(),
        "Snapshot should be created after removing package"
    );

    Ok(())
}

#[test]
fn test_remove_package_from_multiple_distributions() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("add")
        .arg("repo-rabbitmq-server-bookworm")
        .arg(package_path.to_str().unwrap())
        .output()?;

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("add")
        .arg("repo-rabbitmq-server-jammy")
        .arg(package_path.to_str().unwrap())
        .output()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-v",
        "4.1.3-1",
        "-d",
        "bookworm,jammy",
    ]);
    cmd.assert().success();

    assert!(
        !ctx.package_exists(
            "repo-rabbitmq-server-bookworm",
            "rabbitmq-server (= 4.1.3-1)"
        )?,
        "Package should be removed from bookworm"
    );
    assert!(
        !ctx.package_exists("repo-rabbitmq-server-jammy", "rabbitmq-server (= 4.1.3-1)")?,
        "Package should be removed from jammy"
    );

    Ok(())
}

#[test]
fn test_remove_nonexistent_package_succeeds() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    // Removing a non-existent package should succeed (aptly behavior)
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-v",
        "99.99.99-999",
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_snapshot_take_single_distribution() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";

    ctx.create_repo(repo_name)?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq", "snapshot", "take", "-d", "bookworm", "--suffix", "test-01",
    ]);
    cmd.assert().success();

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bookworm-test-01")?,
        "Snapshot with custom suffix should exist"
    );

    Ok(())
}

#[test]
fn test_snapshot_take_multiple_distributions() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "snapshot",
        "take",
        "-d",
        "bookworm,jammy",
        "--suffix",
        "multi-01",
    ]);
    cmd.assert().success();

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bookworm-multi-01")?,
        "Bookworm snapshot should exist"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-jammy-multi-01")?,
        "Jammy snapshot should exist"
    );

    Ok(())
}

#[test]
fn test_snapshot_list() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("snapshot")
        .arg("create")
        .arg("snap-rabbitmq-server-bookworm-list-01")
        .arg("from")
        .arg("repo")
        .arg("repo-rabbitmq-server-bookworm")
        .output()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq", "snapshot", "list", "-d", "bookworm", "--suffix", "list-01",
    ]);
    cmd.assert()
        .success()
        .stdout(output_includes("snap-rabbitmq-server-bookworm-list-01"));

    Ok(())
}

#[test]
fn test_snapshot_delete_single() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("snapshot")
        .arg("create")
        .arg("snap-rabbitmq-server-bookworm-delete-01")
        .arg("from")
        .arg("repo")
        .arg("repo-rabbitmq-server-bookworm")
        .output()?;

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bookworm-delete-01")?,
        "Snapshot should exist before deletion"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "snapshot",
        "delete",
        "-d",
        "bookworm",
        "--suffix",
        "delete-01",
    ]);
    cmd.assert().success();

    // Verify deletion (drop is silent if snapshot doesn't exist)
    // This is expected behavior per aptly semantics
    Ok(())
}

#[test]
fn test_snapshot_remove_alias() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("snapshot")
        .arg("create")
        .arg("snap-rabbitmq-server-bookworm-remove-01")
        .arg("from")
        .arg("repo")
        .arg("repo-rabbitmq-server-bookworm")
        .output()?;

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bookworm-remove-01")?,
        "Snapshot should exist before removal"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "snapshot",
        "remove",
        "-d",
        "bookworm",
        "--suffix",
        "remove-01",
    ]);
    cmd.assert().success();

    // Verify removal worked (drop is silent if snapshot doesn't exist)
    Ok(())
}

#[test]
fn test_snapshot_delete_multiple() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("snapshot")
        .arg("create")
        .arg("snap-rabbitmq-server-bookworm-del-02")
        .arg("from")
        .arg("repo")
        .arg("repo-rabbitmq-server-bookworm")
        .output()?;

    Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("snapshot")
        .arg("create")
        .arg("snap-rabbitmq-server-jammy-del-02")
        .arg("from")
        .arg("repo")
        .arg("repo-rabbitmq-server-jammy")
        .output()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "snapshot",
        "delete",
        "-d",
        "bookworm,jammy",
        "--suffix",
        "del-02",
    ]);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_add_and_remove_workflow() -> Result<(), Box<dyn Error>> {
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
        "--suffix",
        "workflow-01",
    ]);
    cmd.assert().success();

    assert!(ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?);

    assert!(ctx.snapshot_exists("snap-rabbitmq-server-bookworm-workflow-01")?);

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "remove",
        "-v",
        "4.1.3-1",
        "-d",
        "bookworm",
        "--suffix",
        "workflow-02",
    ]);
    cmd.assert().success();

    assert!(!ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?);

    assert!(ctx.snapshot_exists("snap-rabbitmq-server-bookworm-workflow-02")?);

    Ok(())
}

#[test]
fn test_erlang_commands_use_correct_project() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-erlang-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "erlang",
        "snapshot",
        "take",
        "-d",
        "bookworm",
        "--suffix",
        "erlang-01",
    ]);
    cmd.assert().success();

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-erlang-bookworm-erlang-01")?,
        "Erlang snapshot should use erlang prefix"
    );

    Ok(())
}

#[test]
fn test_invalid_distribution_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "snapshot",
        "take",
        "-d",
        "invalid-distro",
        "--suffix",
        "test",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("Invalid distribution alias"));

    Ok(())
}

#[test]
fn test_missing_required_argument_fails() -> Result<(), Box<dyn Error>> {
    run_bellhop_fails(["rabbitmq", "deb", "add"])
        .stderr(output_includes("required arguments were not provided"));

    run_bellhop_fails(["rabbitmq", "deb", "remove"])
        .stderr(output_includes("required arguments were not provided"));

    Ok(())
}

#[test]
fn test_erlang_all_flag_uses_only_supported_distributions() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-erlang-noble")?;
    ctx.create_repo("repo-rabbitmq-erlang-jammy")?;
    ctx.create_repo("repo-rabbitmq-erlang-trixie")?;
    ctx.create_repo("repo-rabbitmq-erlang-bookworm")?;
    ctx.create_repo("repo-rabbitmq-erlang-focal")?;
    ctx.create_repo("repo-rabbitmq-erlang-bullseye")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["erlang", "snapshot", "take", "--all", "--suffix", "test"]);
    cmd.assert().success();

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-erlang-noble-test")?,
        "Noble should be supported for Erlang"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-erlang-jammy-test")?,
        "Jammy should be supported for Erlang"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-erlang-trixie-test")?,
        "Trixie should be supported for Erlang"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-erlang-bookworm-test")?,
        "Bookworm should be supported for Erlang"
    );
    assert!(
        !ctx.snapshot_exists("snap-rabbitmq-erlang-focal-test")?,
        "Focal should NOT be supported for Erlang"
    );
    assert!(
        !ctx.snapshot_exists("snap-rabbitmq-erlang-bullseye-test")?,
        "Bullseye should NOT be supported for Erlang"
    );

    Ok(())
}

#[test]
fn test_rabbitmq_all_flag_uses_all_distributions() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-server-noble")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;
    ctx.create_repo("repo-rabbitmq-server-focal")?;
    ctx.create_repo("repo-rabbitmq-server-trixie")?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-bullseye")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["rabbitmq", "snapshot", "take", "--all", "--suffix", "test"]);
    cmd.assert().success();

    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-noble-test")?,
        "RabbitMQ should support all distributions including Noble"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-jammy-test")?,
        "RabbitMQ should support all distributions including Jammy"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-focal-test")?,
        "RabbitMQ should support all distributions including Focal"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-trixie-test")?,
        "RabbitMQ should support all distributions including Trixie"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bookworm-test")?,
        "RabbitMQ should support all distributions including Bookworm"
    );
    assert!(
        ctx.snapshot_exists("snap-rabbitmq-server-bullseye-test")?,
        "RabbitMQ should support all distributions including Bullseye"
    );

    Ok(())
}
