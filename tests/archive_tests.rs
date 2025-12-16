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
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tar::Builder;
use tempfile::TempDir;
use test_helpers::*;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn create_tar_archive_with_debs(debs: &[&str]) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("packages.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            builder.append_path_with_name(&deb_path, deb)?;
        }
    }

    builder.finish()?;

    Ok((archive_path, temp_dir))
}

fn create_empty_zip_archive() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("empty.zip");
    let zip_file = File::create(&archive_path)?;
    let zip = ZipWriter::new(zip_file);
    zip.finish()?;
    Ok((archive_path, temp_dir))
}

fn create_zip_archive_without_debs() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("no-debs.zip");
    let zip_file = File::create(&archive_path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default();

    zip.start_file("README.txt", options)?;
    zip.write_all(b"Package documentation\nVersion 1.0\n")?;
    zip.start_file("LICENSE", options)?;
    zip.write_all(b"MIT License")?;

    zip.finish()?;

    Ok((archive_path, temp_dir))
}

fn create_zip_archive_with_symlink() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let work_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("symlink.zip");

    // Create a regular .deb file in work directory
    let deb_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    if deb_path.exists() {
        fs::copy(&deb_path, work_dir.path().join("package.deb"))?;
    }

    // Create a symlink in the work directory (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(
            "../../../etc/passwd",
            work_dir.path().join("dangerous_symlink.deb"),
        )?;
    }

    // Use system zip command to create archive with symlink preserved
    #[cfg(unix)]
    {
        let output = Command::new("zip")
            .arg("--symlinks")
            .arg("-r")
            .arg(&archive_path)
            .arg(".")
            .current_dir(work_dir.path())
            .output();

        match output {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                return Err(format!(
                    "Failed to create zip with symlinks: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
                .into());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err("zip command not found - please install zip utility".into());
            }
            Err(e) => return Err(e.into()),
        }
    }

    // On non-Unix systems, just create a regular archive
    #[cfg(not(unix))]
    {
        let zip_file = File::create(&archive_path)?;
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        let mut file = File::open(work_dir.path().join("package.deb"))?;
        zip.start_file("package.deb", options)?;
        std::io::copy(&mut file, &mut zip)?;

        zip.finish()?;
    }

    Ok((archive_path, temp_dir))
}

fn create_corrupted_zip_archive() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("corrupted.zip");
    // Create a file with .zip extension but corrupted content
    fs::write(
        &archive_path,
        b"This is not a valid ZIP file\x00\x01\x02\xFF",
    )?;
    Ok((archive_path, temp_dir))
}

fn create_tar_archive_with_nested_debs(
    debs: &[&str],
) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("nested.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            let nested_path = format!("subdir/packages/{deb}");
            builder.append_path_with_name(&deb_path, nested_path)?;
        }
    }

    builder.finish()?;

    Ok((archive_path, temp_dir))
}

fn create_tar_archive_with_mixed_content(
    debs: &[&str],
) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("mixed.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            builder.append_path_with_name(&deb_path, deb)?;
        }
    }

    let readme_dir = temp_dir.path().join("docs");
    fs::create_dir(&readme_dir)?;
    let readme_path = readme_dir.join("README.txt");
    let mut readme_file = File::create(&readme_path)?;
    readme_file.write_all(b"Package documentation\nVersion 1.0\n")?;

    let license_path = readme_dir.join("LICENSE");
    let mut license_file = File::create(&license_path)?;
    license_file.write_all(b"MIT License")?;

    builder.append_path_with_name(&readme_path, "README.txt")?;
    builder.append_path_with_name(&license_path, "LICENSE")?;

    builder.finish()?;

    Ok((archive_path, temp_dir))
}

fn create_tar_archive_with_deeply_nested_deb() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("deep.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);

    let deb_path = test_package_path("rabbitmq-server_4.1.3-1_all.deb");
    if deb_path.exists() {
        builder.append_path_with_name(&deb_path, "dir1/dir2/dir3/package.deb")?;
    }

    builder.finish()?;

    Ok((archive_path, temp_dir))
}

fn create_empty_tar_archive() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("empty.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);
    builder.finish()?;

    Ok((archive_path, temp_dir))
}

fn create_tar_archive_without_debs() -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("no-debs.tar");
    let tar_file = File::create(&archive_path)?;
    let mut builder = Builder::new(tar_file);

    let readme_dir = temp_dir.path().join("readme");
    fs::create_dir(&readme_dir)?;
    let readme_path = readme_dir.join("README.txt");
    let mut readme_file = File::create(&readme_path)?;
    readme_file.write_all(b"This archive has no .deb files")?;

    builder.append_path_with_name(&readme_path, "README.txt")?;
    builder.finish()?;

    Ok((archive_path, temp_dir))
}

#[test]
fn test_add_tar_archive_with_single_deb() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) =
        create_tar_archive_with_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package should exist in repository"
    );

    Ok(())
}

#[test]
fn test_add_empty_tar_archive_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let (archive_path, _temp_dir) = create_empty_tar_archive()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("No .deb files found in archive"));

    Ok(())
}

#[test]
fn test_add_tar_archive_without_debs_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let (archive_path, _temp_dir) = create_tar_archive_without_debs()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("No .deb files found in archive"));

    Ok(())
}

#[test]
fn test_add_tar_archive_to_multiple_distributions() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;
    ctx.create_repo("repo-rabbitmq-server-jammy")?;

    let (archive_path, _temp_dir) =
        create_tar_archive_with_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
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

    Ok(())
}

#[test]
fn test_erlang_tar_archive_support() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-erlang-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) =
        create_tar_archive_with_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "erlang",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_existing_deb_file_still_works() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

#[test]
fn test_add_tar_archive_with_multiple_debs() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) = create_tar_archive_with_debs(&[
        "rabbitmq-server_4.1.3-1_all.deb",
        "rabbitmq-server_4.1.4-1_all.deb",
        "rabbitmq-server_4.1.5-1_all.deb",
    ])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "First package should exist"
    );
    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.4-1)")?,
        "Second package should exist"
    );
    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.5-1)")?,
        "Third package should exist"
    );

    Ok(())
}

#[test]
fn test_add_tar_archive_with_nested_debs() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) =
        create_tar_archive_with_nested_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Package from nested directory should exist in repository"
    );

    Ok(())
}

#[test]
fn test_add_tar_archive_with_mixed_content() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) = create_tar_archive_with_mixed_content(&[
        "rabbitmq-server_4.1.3-1_all.deb",
        "rabbitmq-server_4.1.4-1_all.deb",
    ])?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "First .deb package should exist"
    );
    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.4-1)")?,
        "Second .deb package should exist"
    );

    Ok(())
}

#[test]
fn test_add_tar_archive_with_deeply_nested_deb_ignored() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let (archive_path, _temp_dir) = create_tar_archive_with_deeply_nested_deb()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("No .deb files found in archive"));

    Ok(())
}

#[test]
fn test_add_real_tar_gz_archive() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let archive_path = test_fixture_path("archives/rabbitmq-4.1.7.tar.gz");

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.7-1)")?,
        "Package from .tar.gz archive should be added"
    );

    Ok(())
}

#[test]
fn test_add_empty_zip_archive_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let (archive_path, _temp_dir) = create_empty_zip_archive()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("No .deb files found in archive"));

    Ok(())
}

#[test]
fn test_add_zip_archive_without_debs_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let (archive_path, _temp_dir) = create_zip_archive_without_debs()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert()
        .failure()
        .stderr(output_includes("No .deb files found in archive"));

    Ok(())
}

#[test]
fn test_add_zip_archive_with_symlink_skipped() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) = match create_zip_archive_with_symlink() {
        Ok(result) => result,
        Err(e) if e.to_string().contains("zip command not found") => {
            eprintln!("Skipping test: {e}");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);
    cmd.assert().success();

    // Should only add the regular .deb file, symlink should be skipped
    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.3-1)")?,
        "Regular .deb file should be added (symlink ignored)"
    );

    Ok(())
}

#[test]
fn test_add_corrupted_zip_archive_fails() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    ctx.create_repo("repo-rabbitmq-server-bookworm")?;

    let (archive_path, _temp_dir) = create_corrupted_zip_archive()?;

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "rabbitmq",
        "deb",
        "add",
        "-p",
        archive_path.to_str().unwrap(),
        "-d",
        "bookworm",
    ]);

    cmd.assert()
        .failure()
        .stderr(output_includes("Failed to extract archive"));

    Ok(())
}
