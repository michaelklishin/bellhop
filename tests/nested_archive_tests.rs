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
use flate2::Compression;
use flate2::write::GzEncoder;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use tar::Builder;
use tempfile::TempDir;
use test_helpers::*;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn create_zip_with_tar_gz_containing_debs(
    debs: &[&str],
) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let tar_gz_path = temp_dir.path().join("packages.tar.gz");
    let zip_path = temp_dir.path().join("archive.zip");

    let tar_gz_file = File::create(&tar_gz_path)?;
    let encoder = GzEncoder::new(tar_gz_file, Compression::default());
    let mut tar_builder = Builder::new(encoder);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            tar_builder.append_path_with_name(&deb_path, deb)?;
        }
    }

    tar_builder.finish()?;

    let zip_file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default();

    let mut tar_gz_file = File::open(&tar_gz_path)?;
    zip.start_file("packages.tar.gz", options)?;
    std::io::copy(&mut tar_gz_file, &mut zip)?;

    zip.finish()?;

    Ok((zip_path, temp_dir))
}

fn create_zip_with_tar_containing_debs(
    debs: &[&str],
) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let tar_path = temp_dir.path().join("packages.tar");
    let zip_path = temp_dir.path().join("archive.zip");

    let tar_file = File::create(&tar_path)?;
    let mut tar_builder = Builder::new(tar_file);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            tar_builder.append_path_with_name(&deb_path, deb)?;
        }
    }

    tar_builder.finish()?;

    let zip_file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default();

    let mut tar_file = File::open(&tar_path)?;
    zip.start_file("packages.tar", options)?;
    std::io::copy(&mut tar_file, &mut zip)?;

    zip.finish()?;

    Ok((zip_path, temp_dir))
}

fn create_zip_with_tgz_containing_debs(
    debs: &[&str],
) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let tgz_path = temp_dir.path().join("packages.tgz");
    let zip_path = temp_dir.path().join("archive.zip");

    let tgz_file = File::create(&tgz_path)?;
    let encoder = GzEncoder::new(tgz_file, Compression::default());
    let mut tar_builder = Builder::new(encoder);

    for deb in debs {
        let deb_path = test_package_path(deb);
        if deb_path.exists() {
            tar_builder.append_path_with_name(&deb_path, deb)?;
        }
    }

    tar_builder.finish()?;

    let zip_file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default();

    let mut tgz_file = File::open(&tgz_path)?;
    zip.start_file("packages.tgz", options)?;
    std::io::copy(&mut tgz_file, &mut zip)?;

    zip.finish()?;

    Ok((zip_path, temp_dir))
}

fn create_zip_with_multiple_tar_gz(
    debs_per_tar: &[&[&str]],
) -> Result<(PathBuf, TempDir), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let zip_path = temp_dir.path().join("archive.zip");
    let zip_file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default();

    for (idx, debs) in debs_per_tar.iter().enumerate() {
        let tar_gz_path = temp_dir.path().join(format!("packages-{}.tar.gz", idx));
        let tar_gz_file = File::create(&tar_gz_path)?;
        let encoder = GzEncoder::new(tar_gz_file, Compression::default());
        let mut tar_builder = Builder::new(encoder);

        for deb in *debs {
            let deb_path = test_package_path(deb);
            if deb_path.exists() {
                tar_builder.append_path_with_name(&deb_path, deb)?;
            }
        }

        tar_builder.finish()?;

        let mut tar_gz_file = File::open(&tar_gz_path)?;
        zip.start_file(format!("packages-{}.tar.gz", idx), options)?;
        std::io::copy(&mut tar_gz_file, &mut zip)?;
    }

    zip.finish()?;

    Ok((zip_path, temp_dir))
}

#[test]
fn test_add_zip_with_nested_tar_gz() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) =
        create_zip_with_tar_gz_containing_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

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
        "Package should be extracted from ZIP -> tar.gz -> .deb"
    );

    Ok(())
}

#[test]
fn test_add_zip_with_nested_tar() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) =
        create_zip_with_tar_containing_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

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
        "Package should be extracted from ZIP -> tar -> .deb"
    );

    Ok(())
}

#[test]
fn test_add_zip_with_nested_tgz() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) =
        create_zip_with_tgz_containing_debs(&["rabbitmq-server_4.1.3-1_all.deb"])?;

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
        "Package should be extracted from ZIP -> tgz -> .deb"
    );

    Ok(())
}

#[test]
fn test_add_zip_with_multiple_nested_tar_gz() -> Result<(), Box<dyn Error>> {
    if !test_packages_available() {
        eprintln!("Skipping test: test packages not available");
        return Ok(());
    }

    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-server-bookworm";
    ctx.create_repo(repo_name)?;

    let (archive_path, _temp_dir) = create_zip_with_multiple_tar_gz(&[
        &["rabbitmq-server_4.1.3-1_all.deb"],
        &["rabbitmq-server_4.1.4-1_all.deb"],
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
        "First package from first tar.gz should exist"
    );
    assert!(
        ctx.package_exists(repo_name, "rabbitmq-server (= 4.1.4-1)")?,
        "Second package from second tar.gz should exist"
    );

    Ok(())
}

#[test]
fn test_add_real_world_erlang_zip_with_nested_tar_gz() -> Result<(), Box<dyn Error>> {
    let ctx = AptlyTestContext::new()?;
    let repo_name = "repo-rabbitmq-erlang-noble";
    ctx.create_repo(repo_name)?;

    let fixture_path = test_fixture_path("erlang-27.3.4.6-ubuntu-noble.zip");

    if !fixture_path.exists() {
        eprintln!(
            "Skipping test: fixture not available at {}",
            fixture_path.display()
        );
        return Ok(());
    }

    let mut cmd = Command::new(cargo::cargo_bin!("bellhop"));
    cmd.env("APTLY_CONFIG", ctx.config_path.to_str().unwrap());
    cmd.args([
        "erlang",
        "deb",
        "add",
        "-p",
        fixture_path.to_str().unwrap(),
        "-d",
        "noble",
    ]);
    cmd.assert().success();

    let packages = ctx.list_packages(repo_name)?;
    assert!(
        !packages.is_empty(),
        "Failed extraction of a wrapped Actions-produced archive: ZIP -> tar.gz -> .deb"
    );
    assert!(
        packages.iter().any(|p| p.contains("erlang-base")),
        "erlang-base package should be in the repository"
    );

    Ok(())
}
