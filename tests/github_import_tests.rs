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
fn test_rabbitmq_import_from_github_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["rabbitmq", "deb", "import-from-github", "--help"])
        .stdout(output_includes(
            "Import .deb packages from a GitHub release",
        ))
        .stdout(output_includes("--github-release-url"));
    Ok(())
}

#[test]
fn test_cli_tools_import_from_github_help() -> Result<(), Box<dyn Error>> {
    run_bellhop_succeeds(["cli-tools", "deb", "import-from-github", "--help"])
        .stdout(output_includes(
            "Import .deb packages from a GitHub release",
        ))
        .stdout(output_includes("--github-release-url"));
    Ok(())
}

#[test]
fn test_erlang_does_not_have_import_from_github() -> Result<(), Box<dyn Error>> {
    run_bellhop_fails(["erlang", "deb", "import-from-github", "--help"]);
    Ok(())
}

#[test]
fn test_import_from_github_requires_url() -> Result<(), Box<dyn Error>> {
    run_bellhop_fails(["rabbitmq", "deb", "import-from-github", "-d", "bookworm"])
        .stderr(output_includes("required arguments were not provided"));
    Ok(())
}

#[test]
fn test_import_from_github_invalid_url() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "import-from-github",
        "--github-release-url",
        "https://not-github.com/foo/bar",
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("Invalid GitHub release URL"));

    Ok(())
}

#[test]
#[ignore]
fn test_import_rabbitmq_server_from_github() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "import-from-github",
        "--github-release-url",
        "https://github.com/rabbitmq/rabbitmq-server/releases/tag/v4.2.3",
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(
            "repo-rabbitmq-server-bookworm",
            "rabbitmq-server (= 4.2.3-1)"
        )?,
        "Package should be imported from GitHub release"
    );

    Ok(())
}

#[test]
#[ignore]
fn test_import_cli_tools_from_github() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-cli-bookworm")?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "cli-tools",
        "deb",
        "import-from-github",
        "--github-release-url",
        "https://github.com/rabbitmq/rabbitmqadmin-ng/releases/tag/v2.25.0",
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    let packages = ctx.list_packages("repo-rabbitmq-cli-bookworm")?;
    assert!(
        !packages.is_empty(),
        "Should import amd64 package from GitHub release"
    );

    Ok(())
}
