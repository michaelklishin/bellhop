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
#![allow(dead_code)]

use assert_cmd::assert::Assert;
use assert_cmd::cargo;
use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

type CommandRunResult = Result<(), Box<dyn Error>>;

/// Test context that manages temporary aptly environment
pub struct AptlyTestContext {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl AptlyTestContext {
    /// Create a new aptly test context with temporary directory and config
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("aptly.conf");

        let config_content = format!(
            r#"{{
  "rootDir": "{}",
  "downloadConcurrency": 4,
  "downloadSpeedLimit": 0,
  "architectures": ["amd64", "arm64", "armel", "armhf", "i386"],
  "dependencyFollowSuggests": false,
  "dependencyFollowRecommends": false,
  "dependencyFollowAllVariants": false,
  "dependencyFollowSource": false,
  "gpgDisableSign": false,
  "gpgDisableVerify": false,
  "downloadSourcePackages": false,
  "ppaDistributorID": "ubuntu",
  "ppaCodename": "",
  "S3PublishEndpoints": {{}},
  "SwiftPublishEndpoints": {{}}
}}"#,
            temp_dir.path().display()
        );

        fs::write(&config_path, config_content)?;

        Ok(AptlyTestContext {
            temp_dir,
            config_path,
        })
    }

    /// Get the config path as a string
    pub fn config_arg(&self) -> String {
        format!("-config={}", self.config_path.display())
    }

    /// Create an aptly repository
    pub fn create_repo(&self, repo_name: &str) -> CommandRunResult {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("repo")
            .arg("create")
            .arg(repo_name)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to create repo: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    /// List packages in a repository
    pub fn list_packages(&self, repo_name: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("repo")
            .arg("show")
            .arg("-with-packages")
            .arg(repo_name)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to list packages: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = stdout
            .lines()
            .skip_while(|line| !line.contains("Packages:"))
            .skip(1) // Skip the "Packages:" line itself
            .filter(|line| !line.is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(packages)
    }

    /// Check if a package exists in a repository
    pub fn package_exists(
        &self,
        repo_name: &str,
        package_query: &str,
    ) -> Result<bool, Box<dyn Error>> {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("repo")
            .arg("search")
            .arg(repo_name)
            .arg(package_query)
            .output()?;

        // aptly search returns success even if no results, but empty stdout
        Ok(output.status.success() && !output.stdout.is_empty())
    }

    /// List snapshots matching a pattern
    pub fn list_snapshots(&self, pattern: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("snapshot")
            .arg("list")
            .arg("-raw")
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to list snapshots: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let snapshots: Vec<String> = stdout
            .lines()
            .filter(|line| line.contains(pattern))
            .map(|line| line.trim().to_string())
            .collect();

        Ok(snapshots)
    }

    /// Check if a snapshot exists
    pub fn snapshot_exists(&self, snapshot_name: &str) -> Result<bool, Box<dyn Error>> {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("snapshot")
            .arg("show")
            .arg(snapshot_name)
            .output()?;

        Ok(output.status.success())
    }

    /// Get the number of packages in a snapshot
    pub fn snapshot_package_count(&self, snapshot_name: &str) -> Result<usize, Box<dyn Error>> {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("snapshot")
            .arg("show")
            .arg("-with-packages")
            .arg(snapshot_name)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to show snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout
            .lines()
            .find(|line| line.contains("Number of packages:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|num| num.trim().parse::<usize>().ok())
            .unwrap_or(0);

        Ok(count)
    }

    /// Create an initial publish for a repository
    pub fn create_initial_publish(
        &self,
        prefix: &str,
        family: &str,
        distribution: &str,
    ) -> CommandRunResult {
        let repo_name = format!("repo-{prefix}-{distribution}");
        let snapshot_name = format!("snap-{prefix}-{distribution}-init");
        let publish_prefix = format!("{prefix}/{family}/{distribution}");

        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("snapshot")
            .arg("create")
            .arg(&snapshot_name)
            .arg("from")
            .arg("repo")
            .arg(&repo_name)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to create initial snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("publish")
            .arg("snapshot")
            .arg(format!("-distribution={distribution}"))
            .arg(&snapshot_name)
            .arg(&publish_prefix)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to publish initial snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    /// Check if a specific snapshot is currently active for a published repository
    pub fn published_snapshot_is_active(
        &self,
        prefix: &str,
        family: &str,
        distribution: &str,
        snapshot_name: &str,
    ) -> Result<bool, Box<dyn Error>> {
        let output = Command::new("aptly")
            .arg(self.config_arg())
            .arg("publish")
            .arg("list")
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to list published repositories: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let publish_prefix = format!("{prefix}/{family}/{distribution}");

        for line in stdout.lines() {
            if line.contains(&publish_prefix) && line.contains(snapshot_name) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Run bellhop command and expect success
pub fn run_bellhop_succeeds<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.args(args).assert().success()
}

/// Run bellhop command and expect failure
pub fn run_bellhop_fails<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.args(args).assert().failure()
}

/// Create a predicate for checking if output contains a string
pub fn output_includes(content: &str) -> predicates::str::ContainsPredicate {
    predicate::str::contains(content)
}

/// Get path to a test package file
pub fn test_package_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("debs")
        .join(filename)
}

/// Get path to a test fixture file in tests/fixtures/
pub fn test_fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}
