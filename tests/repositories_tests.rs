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
fn test_repositories_setup_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["repositories", "set-up", "--help"])
        .stdout(output_includes("Create all expected aptly repositories"));
    Ok(())
}

#[test]
fn test_repositories_setup_alias() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["repositories", "setup", "--help"])
        .stdout(output_includes("Create all expected aptly repositories"));
    Ok(())
}

#[test]
fn test_repositories_setup_creates_all_repos() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["repositories", "set-up"]);
    cmd.assert().success();

    let output = Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("list")
        .arg("-raw")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let repos: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    // 7 RabbitMQ + 5 Erlang + 6 CLI tools = 18
    assert_eq!(repos.len(), 18, "Should create all 18 expected repos");

    assert!(
        repos.contains(&"repo-rabbitmq-server-resolute"),
        "Should create RabbitMQ repo for resolute"
    );
    assert!(!repos.contains(&"repo-rabbitmq-cli-resolute"));
    assert!(repos.contains(&"repo-rabbitmq-erlang-resolute"));

    for dist in ["noble", "jammy", "focal", "trixie", "bookworm", "bullseye"] {
        assert!(
            repos.contains(&format!("repo-rabbitmq-server-{dist}").as_str()),
            "Should create RabbitMQ repo for {dist}"
        );
    }
    for dist in ["noble", "jammy", "trixie", "bookworm"] {
        assert!(
            repos.contains(&format!("repo-rabbitmq-erlang-{dist}").as_str()),
            "Should create Erlang repo for {dist}"
        );
    }
    for dist in ["noble", "jammy", "focal", "trixie", "bookworm", "bullseye"] {
        assert!(
            repos.contains(&format!("repo-rabbitmq-cli-{dist}").as_str()),
            "Should create CLI tools repo for {dist}"
        );
    }

    Ok(())
}

#[test]
fn test_repositories_setup_is_idempotent() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    let mut cmd1 = Command::new(cargo::cargo_bin!("bellhop"));
    cmd1.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd1.args(["repositories", "set-up"]);
    cmd1.assert().success();

    let mut cmd2 = Command::new(cargo::cargo_bin!("bellhop"));
    cmd2.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd2.args(["repositories", "set-up"]);
    cmd2.assert().success();

    let output = Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("list")
        .arg("-raw")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let repos: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert_eq!(repos.len(), 18, "Should still have exactly 18 repos");

    Ok(())
}

#[test]
fn test_repositories_setup_creates_only_missing() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;

    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-erlang-trixie")?;
    ctx.create_repo("repo-rabbitmq-cli-noble")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args(["repositories", "set-up"]);
    cmd.assert().success();

    let output = Command::new("aptly")
        .arg(ctx.config_arg())
        .arg("repo")
        .arg("list")
        .arg("-raw")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let repos: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert_eq!(
        repos.len(),
        18,
        "Should have all 18 repos after filling in missing ones"
    );

    Ok(())
}
