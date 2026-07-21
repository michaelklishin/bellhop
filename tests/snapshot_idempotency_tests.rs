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

const REPO: &str = "repo-rabbitmq-server-bookworm";

fn snapshot_name_for_today() -> String {
    let date = Local::now().format("%d-%b-%y").to_string();
    format!("snap-rabbitmq-server-bookworm-{date}")
}

fn bellhop(ctx: &AptlyTestContext, args: &[&str]) -> Command {
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(args);
    cmd
}

/// Goes through `aptly` rather than `bellhop` so that no snapshot is taken and the
/// repository ends up ahead of the existing snapshot
fn add_package_directly(ctx: &AptlyTestContext, filename: &str) -> Result<(), Box<dyn Error>> {
    let package_path = test_package_path(filename);
    let output = Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("add")
        .arg(REPO)
        .arg(package_path.to_str().unwrap())
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to add package directly: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

fn publish_snapshot(ctx: &AptlyTestContext, snapshot_name: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("publish")
        .arg("snapshot")
        .arg("-skip-signing")
        .arg("-distribution=bookworm")
        .arg(snapshot_name)
        .arg("rabbitmq-server/debian/bookworm")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to publish snapshot: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

fn setup_with_one_package() -> Result<AptlyTestContext, Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo(REPO)?;

    let package_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    bellhop(
        &ctx,
        &[
            "rabbitmq",
            "deb",
            "add",
            "-p",
            package_path.to_str().unwrap(),
            "-d",
            "bookworm",
        ],
    )
    .assert()
    .success();

    Ok(ctx)
}

#[test]
fn test_take_snapshot_succeeds_when_snapshot_is_absent() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo(REPO)?;

    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .success();

    assert!(
        ctx.snapshot_exists(&snapshot_name_for_today())?,
        "Snapshot should have been created"
    );

    Ok(())
}

#[test]
fn test_take_snapshot_is_idempotent_when_repo_is_unchanged() -> Result<(), Box<dyn Error>> {
    let ctx = setup_with_one_package()?;
    let snapshot = snapshot_name_for_today();
    let count_before = ctx.snapshot_package_count(&snapshot)?;

    // The snapshot already exists here, `deb add` created it
    for _ in 0..3 {
        bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
            .assert()
            .success();
    }

    assert!(
        ctx.snapshot_exists(&snapshot)?,
        "Snapshot should still exist"
    );
    assert_eq!(
        count_before,
        ctx.snapshot_package_count(&snapshot)?,
        "Repeated takes should leave the snapshot unchanged"
    );

    Ok(())
}

#[test]
fn test_take_snapshot_reports_that_there_is_nothing_to_do() -> Result<(), Box<dyn Error>> {
    let ctx = setup_with_one_package()?;

    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .success()
        .stderr(output_includes("nothing to do"));

    Ok(())
}

#[test]
fn test_take_snapshot_refreshes_a_stale_snapshot() -> Result<(), Box<dyn Error>> {
    let ctx = setup_with_one_package()?;
    let snapshot = snapshot_name_for_today();
    let count_before = ctx.snapshot_package_count(&snapshot)?;

    add_package_directly(&ctx, "rabbitmq-server_4.1.4-1_all.deb")?;

    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .success();

    assert_eq!(
        count_before + 1,
        ctx.snapshot_package_count(&snapshot)?,
        "A stale snapshot should be refreshed to match the repository"
    );

    Ok(())
}

#[test]
fn test_take_snapshot_leaves_no_temporary_snapshot_behind() -> Result<(), Box<dyn Error>> {
    let ctx = setup_with_one_package()?;

    // Covers both branches: repository unchanged, then repository ahead of the snapshot
    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .success();
    add_package_directly(&ctx, "rabbitmq-server_4.1.4-1_all.deb")?;
    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .success();

    let leftovers = ctx.list_snapshots("-bellhop-tmp")?;
    assert!(
        leftovers.is_empty(),
        "Temporary snapshots should be cleaned up, found: {leftovers:?}"
    );

    Ok(())
}

#[test]
fn test_take_snapshot_refuses_to_replace_a_published_stale_snapshot() -> Result<(), Box<dyn Error>>
{
    let ctx = setup_with_one_package()?;
    let snapshot = snapshot_name_for_today();
    publish_snapshot(&ctx, &snapshot)?;

    let count_before = ctx.snapshot_package_count(&snapshot)?;
    add_package_directly(&ctx, "rabbitmq-server_4.1.4-1_all.deb")?;

    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .failure()
        .stderr(output_includes("currently published"));

    assert_eq!(
        count_before,
        ctx.snapshot_package_count(&snapshot)?,
        "A published snapshot must not be altered"
    );

    Ok(())
}

#[test]
fn test_take_snapshot_is_idempotent_when_published_and_unchanged() -> Result<(), Box<dyn Error>> {
    let ctx = setup_with_one_package()?;
    let snapshot = snapshot_name_for_today();
    publish_snapshot(&ctx, &snapshot)?;

    // Being published only blocks the replacement path, not a no-op take
    bellhop(&ctx, &["rabbitmq", "snapshot", "take", "-d", "bookworm"])
        .assert()
        .success();

    assert!(
        ctx.snapshot_exists(&snapshot)?,
        "Snapshot should still exist"
    );

    Ok(())
}
