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

fn add(ctx: &AptlyTestContext, filename: &str, extra: &[&str]) -> Command {
    let path = test_package_path(filename);
    let mut args = vec!["rabbitmq", "deb", "add", "-p", path.to_str().unwrap()];
    args.extend_from_slice(extra);
    bellhop(ctx, &args)
}

fn setup_published_repo() -> Result<AptlyTestContext, Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo(REPO)?;
    add(&ctx, "rabbitmq-server_4.1.3-1_all.deb", &["-d", "bookworm"])
        .assert()
        .success();
    bellhop(&ctx, &["rabbitmq", "deb", "publish", "-d", "bookworm"])
        .assert()
        .success();
    Ok(ctx)
}

// A second same-day import into an already published repo used to fail confusingly: the fixed-name
// snapshot could neither be dropped nor recreated. It must now refuse cleanly and point at --suffix.
#[test]
fn test_add_refuses_to_replace_a_published_stale_snapshot() -> Result<(), Box<dyn Error>> {
    let ctx = setup_published_repo()?;
    let snapshot = snapshot_name_for_today();
    let count_before = ctx.snapshot_package_count(&snapshot)?;

    add(&ctx, "rabbitmq-server_4.1.4-1_all.deb", &["-d", "bookworm"])
        .assert()
        .failure()
        .stderr(output_includes("currently published"));

    // The package still reaches the repository, only the snapshot is left untouched
    assert!(
        ctx.package_exists(REPO, "rabbitmq-server (= 4.1.4-1)")?,
        "The package should have been added to the repository"
    );
    assert_eq!(
        count_before,
        ctx.snapshot_package_count(&snapshot)?,
        "A published snapshot must not be altered by add"
    );

    Ok(())
}

// The identical-contents case must not trip the published-snapshot guard: re-adding the same
// package leaves the repo unchanged, so there is nothing to replace.
#[test]
fn test_add_is_a_noop_when_published_and_unchanged() -> Result<(), Box<dyn Error>> {
    let ctx = setup_published_repo()?;
    let snapshot = snapshot_name_for_today();

    add(&ctx, "rabbitmq-server_4.1.3-1_all.deb", &["-d", "bookworm"])
        .assert()
        .success()
        .stderr(output_includes("nothing to do"));

    assert!(
        ctx.published_snapshot_is_active("rabbitmq-server", "debian", "bookworm", &snapshot)?,
        "The snapshot should remain published and unchanged"
    );

    Ok(())
}

// Without publication in the way, a same-day import refreshes the fixed-name snapshot in place.
#[test]
fn test_add_refreshes_snapshot_when_unpublished() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo(REPO)?;
    add(&ctx, "rabbitmq-server_4.1.3-1_all.deb", &["-d", "bookworm"])
        .assert()
        .success();
    let snapshot = snapshot_name_for_today();
    let count_before = ctx.snapshot_package_count(&snapshot)?;

    add(&ctx, "rabbitmq-server_4.1.4-1_all.deb", &["-d", "bookworm"])
        .assert()
        .success();

    assert_eq!(
        count_before + 1,
        ctx.snapshot_package_count(&snapshot)?,
        "An unpublished snapshot should be refreshed to match the repository"
    );

    Ok(())
}

// The escape hatch the refusal suggests must actually work end to end: an import under a suffix,
// then a publish of that same suffix.
#[test]
fn test_suffix_import_then_publish_resolves_collision() -> Result<(), Box<dyn Error>> {
    let ctx = setup_published_repo()?;
    let today = snapshot_name_for_today();
    let suffixed = "snap-rabbitmq-server-bookworm-next";

    add(
        &ctx,
        "rabbitmq-server_4.1.4-1_all.deb",
        &["-d", "bookworm", "--suffix", "next"],
    )
    .assert()
    .success();

    assert!(
        ctx.snapshot_exists(suffixed)?,
        "The suffixed snapshot should have been created"
    );
    assert!(
        ctx.published_snapshot_is_active("rabbitmq-server", "debian", "bookworm", &today)?,
        "The original snapshot should still be published before the suffixed publish"
    );

    bellhop(
        &ctx,
        &[
            "rabbitmq", "deb", "publish", "-d", "bookworm", "--suffix", "next",
        ],
    )
    .assert()
    .success();

    assert!(
        ctx.published_snapshot_is_active("rabbitmq-server", "debian", "bookworm", suffixed)?,
        "The suffixed snapshot should now be the published one"
    );
    assert!(
        ctx.package_exists(REPO, "rabbitmq-server (= 4.1.4-1)")?,
        "The newly imported package should be present in the repository"
    );
    // The escape hatch must not destroy the snapshot it replaced
    assert!(
        ctx.snapshot_exists(&today)?,
        "The original snapshot should be left intact, only no longer published"
    );

    Ok(())
}

// Removal shares the same snapshot path as add, so it inherits the same published-snapshot guard.
#[test]
fn test_remove_refuses_to_replace_a_published_stale_snapshot() -> Result<(), Box<dyn Error>> {
    let ctx = setup_published_repo()?;
    let snapshot = snapshot_name_for_today();
    let count_before = ctx.snapshot_package_count(&snapshot)?;

    bellhop(
        &ctx,
        &[
            "rabbitmq", "deb", "remove", "-v", "4.1.3-1", "-d", "bookworm",
        ],
    )
    .assert()
    .failure()
    .stderr(output_includes("currently published"));

    assert_eq!(
        count_before,
        ctx.snapshot_package_count(&snapshot)?,
        "A published snapshot must not be altered by remove"
    );

    Ok(())
}

// publish with no --suffix must keep targeting today's snapshot, the pre-existing behavior.
#[test]
fn test_publish_without_suffix_targets_todays_snapshot() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo(REPO)?;
    add(&ctx, "rabbitmq-server_4.1.3-1_all.deb", &["-d", "bookworm"])
        .assert()
        .success();

    bellhop(&ctx, &["rabbitmq", "deb", "publish", "-d", "bookworm"])
        .assert()
        .success();

    assert!(
        ctx.published_snapshot_is_active(
            "rabbitmq-server",
            "debian",
            "bookworm",
            &snapshot_name_for_today()
        )?,
        "publish without a suffix should target today's snapshot"
    );

    Ok(())
}
