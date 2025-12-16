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
fn test_erlang_add_trixie_zip_archive() -> Result<(), Box<dyn Error>> {
    let archive_path = test_fixture_path("archives/erlang-27.3.4.6-debian-trixie.zip");
    if !archive_path.exists() {
        eprintln!("Skipping test: Trixie archive not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-erlang-trixie";
    ctx.create_repo(repo_name)?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "erlang",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "trixie",
    ]);
    cmd.assert().success();

    let packages = ctx.list_packages(repo_name)?;
    assert!(
        !packages.is_empty(),
        "Repository should contain Erlang packages"
    );
    assert!(
        packages.iter().any(|p| p.contains("erlang-base")),
        "Should contain erlang-base package"
    );
    assert!(
        packages.iter().any(|p| p.contains("27.3.4.6")),
        "Should contain version 27.3.4.6"
    );

    Ok(())
}

#[test]
fn test_erlang_remove_trixie_by_version() -> Result<(), Box<dyn Error>> {
    let archive_path = test_fixture_path("archives/erlang-27.3.4.6-debian-trixie.zip");
    if !archive_path.exists() {
        eprintln!("Skipping test: Trixie archive not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-erlang-trixie";
    ctx.create_repo(repo_name)?;

    let mut add_cmd = Command::new(cargo::cargo_bin!("bellhop"));
    add_cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    add_cmd.args([
        "erlang",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "trixie",
    ]);
    add_cmd.assert().success();

    let mut remove_cmd = Command::new(cargo::cargo_bin!("bellhop"));
    remove_cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    remove_cmd.args([
        "erlang",
        "deb",
        "remove",
        "-v",
        "1:27.3.4.6-1",
        "-d",
        "trixie",
    ]);
    remove_cmd.assert().success();

    let packages_after = ctx.list_packages(repo_name)?;
    assert!(
        packages_after.is_empty(),
        "Repository should be empty after removing all packages with version 1:27.3.4.6-1"
    );

    Ok(())
}
