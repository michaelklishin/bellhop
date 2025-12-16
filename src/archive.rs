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
use crate::errors::BellhopError;
use flate2::read::GzDecoder;
use log::{debug, info};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tar::Archive;
use tempfile::TempDir;
use zip::ZipArchive;

pub enum PackageSource {
    SingleDeb(PathBuf),
    Archive {
        deb_files: Vec<PathBuf>,
        _temp_dir: TempDir,
    },
}

pub fn process_package_file(package_file_path: &Path) -> Result<PackageSource, BellhopError> {
    let file_name = package_file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if file_name.ends_with(".deb") {
        debug!("Detected .deb file: {}", package_file_path.display());
        return Ok(PackageSource::SingleDeb(package_file_path.to_path_buf()));
    }

    if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        info!("Detected .tar.gz archive: {}", package_file_path.display());
        return extract_tar_gz(package_file_path);
    }

    if file_name.ends_with(".tar") {
        info!("Detected .tar archive: {}", package_file_path.display());
        return extract_tar(package_file_path);
    }

    if file_name.ends_with(".zip") {
        info!("Detected .zip archive: {}", package_file_path.display());
        return extract_zip(package_file_path);
    }

    debug!("Assuming .deb file: {}", package_file_path.display());
    Ok(PackageSource::SingleDeb(package_file_path.to_path_buf()))
}

fn extract_tar_gz(archive_path: &Path) -> Result<PackageSource, BellhopError> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let archive = Archive::new(decoder);

    extract_and_find_debs(archive, archive_path)
}

fn extract_tar(archive_path: &Path) -> Result<PackageSource, BellhopError> {
    let file = File::open(archive_path)?;
    let archive = Archive::new(file);

    extract_and_find_debs(archive, archive_path)
}

fn extract_zip(archive_path: &Path) -> Result<PackageSource, BellhopError> {
    let file = File::open(archive_path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| BellhopError::ArchiveExtractionFailed(e.to_string()))?;

    let temp_dir = TempDir::new()?;
    let extract_path = temp_dir.path();

    info!("Extracting ZIP archive to: {}", extract_path.display());

    // Due to a zip crate limitation,
    // all files are created with default permissions (0666 & umask).

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| BellhopError::ArchiveExtractionFailed(e.to_string()))?;

        let Some(entry_name) = entry.enclosed_name() else {
            debug!("Skipping entry with unsafe path");
            continue;
        };

        // Skip symlinks for security
        if entry.is_symlink() {
            debug!("Skipping symlink: {}", entry_name.display());
            continue;
        }

        let outpath = extract_path.join(entry_name);

        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut entry, &mut outfile)?;
        }
    }

    finalize_archive_extraction(temp_dir, archive_path)
}

fn extract_and_find_debs<R: Read>(
    mut archive: Archive<R>,
    archive_path: &Path,
) -> Result<PackageSource, BellhopError> {
    let temp_dir = TempDir::new()?;
    let extract_path = temp_dir.path();

    archive.set_preserve_permissions(false);
    archive.set_preserve_mtime(false);
    archive.set_unpack_xattrs(false);

    info!("Extracting archive to: {}", extract_path.display());
    archive
        .unpack(extract_path)
        .map_err(|e| BellhopError::ArchiveExtractionFailed(e.to_string()))?;

    finalize_archive_extraction(temp_dir, archive_path)
}

fn finalize_archive_extraction(
    temp_dir: TempDir,
    archive_path: &Path,
) -> Result<PackageSource, BellhopError> {
    extract_nested_tar_archives(temp_dir.path())?;

    let deb_files = find_deb_files(temp_dir.path())?;

    if deb_files.is_empty() {
        return Err(BellhopError::NoDebFilesInArchive {
            path: archive_path.to_path_buf(),
        });
    }

    info!("Found {} .deb files in archive", deb_files.len());
    for deb in &deb_files {
        debug!("  - {}", deb.display());
    }

    Ok(PackageSource::Archive {
        deb_files,
        _temp_dir: temp_dir,
    })
}

fn extract_nested_tar_archives(dir: &Path) -> Result<(), BellhopError> {
    let tar_archives = find_tar_archives(dir)?;

    for tar_path in tar_archives {
        info!("Extracting nested archive: {}", tar_path.display());

        let file_name = tar_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            let file = File::open(&tar_path)?;
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            extract_tar_to_same_dir(&mut archive, &tar_path)?;
        } else if file_name.ends_with(".tar") {
            let file = File::open(&tar_path)?;
            let mut archive = Archive::new(file);
            extract_tar_to_same_dir(&mut archive, &tar_path)?;
        }

        fs::remove_file(&tar_path)?;
    }

    Ok(())
}

fn extract_tar_to_same_dir<R: Read>(
    archive: &mut Archive<R>,
    tar_path: &Path,
) -> Result<(), BellhopError> {
    let parent_dir = tar_path
        .parent()
        .ok_or_else(|| BellhopError::ArchiveExtractionFailed("Invalid tar path".to_string()))?;

    archive.set_preserve_permissions(false);
    archive.set_preserve_mtime(false);
    archive.set_unpack_xattrs(false);

    archive
        .unpack(parent_dir)
        .map_err(|e| BellhopError::ArchiveExtractionFailed(e.to_string()))?;

    Ok(())
}

fn find_tar_archives(dir: &Path) -> Result<Vec<PathBuf>, BellhopError> {
    let mut tar_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_file()
            && path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                n.ends_with(".tar.gz") || n.ends_with(".tgz") || n.ends_with(".tar")
            })
        {
            tar_files.push(path);
        }
    }

    Ok(tar_files)
}

fn find_deb_files(root: &Path) -> Result<Vec<PathBuf>, BellhopError> {
    const MAX_DEPTH: usize = 2;

    let mut deb_files = Vec::new();
    let mut to_visit = vec![(root.to_path_buf(), 0)];

    while let Some((dir, depth)) = to_visit.pop() {
        if depth > MAX_DEPTH {
            continue;
        }

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_file() && path.extension().is_some_and(|ext| ext == "deb") {
                deb_files.push(path);
            } else if file_type.is_dir() {
                to_visit.push((path, depth + 1));
            }
        }
    }

    Ok(deb_files)
}

pub fn extract_versions_from_debs(deb_files: &[PathBuf]) -> Result<Vec<String>, BellhopError> {
    deb_files
        .iter()
        .map(|deb_path| {
            let file_name = deb_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    BellhopError::ArchiveExtractionFailed(format!(
                        "Invalid .deb filename: {}",
                        deb_path.display()
                    ))
                })?;
            extract_version_from_filename(file_name)
        })
        .collect()
}

pub fn extract_version_from_filename(filename: &str) -> Result<String, BellhopError> {
    if !filename.ends_with(".deb") {
        return Err(BellhopError::ArchiveExtractionFailed(format!(
            "Not a .deb file: {filename}"
        )));
    }

    let parts: Vec<&str> = filename.trim_end_matches(".deb").rsplitn(3, '_').collect();

    if parts.len() < 3 {
        return Err(BellhopError::ArchiveExtractionFailed(format!(
            "Invalid .deb filename format: {filename}"
        )));
    }

    Ok(parts[1].to_string())
}
