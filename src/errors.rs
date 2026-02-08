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
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BellhopError {
    #[error("Unknown command '{first} {second} {third}'")]
    UnknownCommand {
        first: String,
        second: String,
        third: String,
    },

    #[error("Package file does not exist at {path}")]
    PackageFileNotFound { path: PathBuf },

    #[error("Invalid distribution alias: {alias}")]
    InvalidDistribution { alias: String },

    #[error("Required argument '{argument}' is missing")]
    MissingArgument { argument: String },

    #[error("aptly command failed: {command}\nStderr: {stderr}")]
    #[allow(dead_code)]
    AptlyCommandFailed { command: String, stderr: String },

    #[error(
        "aptly command failed with status {status}: {command}\nStdout: {stdout}\nStderr: {stderr}"
    )]
    AptlyNonZeroExit {
        command: String,
        status: i32,
        stdout: String,
        stderr: String,
    },

    #[error("Run into an I/O error when executing aptly: {0}")]
    IoError(#[from] io::Error),

    #[error("No .deb files found in archive: {path}")]
    NoDebFilesInArchive { path: PathBuf },

    #[error("Failed to extract archive: {0}")]
    ArchiveExtractionFailed(String),

    #[error("Not a .deb file: {filename}")]
    InvalidDebFilename { filename: String },

    #[error("Malformed .deb filename (expected format: package_version_arch.deb): {filename}")]
    MalformedDebFilename { filename: String },

    #[error(
        "aptly executable not found. Please install aptly first: https://www.aptly.info/download/"
    )]
    AptlyNotFound,

    #[error("Invalid GitHub release URL: {url}")]
    InvalidGitHubReleaseUrl { url: String },

    #[error("GitHub API request failed: {message}")]
    GitHubApiFailed { message: String },

    #[error("No assets matching pattern '{pattern}' in the GitHub release")]
    NoAssetsInRelease { pattern: String },

    #[error("Failed to download {url}: {message}")]
    DownloadFailed { url: String, message: String },

    #[error("Watcher error: {0}")]
    WatcherError(String),
}

#[repr(i32)]
pub enum ExitCode {
    Ok = 0,
    Usage = 64,
    DataErr = 65,
    Software = 70,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

pub fn map_error_to_exit_code(error: &BellhopError) -> ExitCode {
    match error {
        BellhopError::UnknownCommand { .. } => ExitCode::Usage,
        BellhopError::MissingArgument { .. } => ExitCode::Usage,
        BellhopError::InvalidDistribution { .. } => ExitCode::DataErr,
        BellhopError::PackageFileNotFound { .. } => ExitCode::DataErr,
        BellhopError::NoDebFilesInArchive { .. } => ExitCode::DataErr,
        BellhopError::InvalidDebFilename { .. } => ExitCode::DataErr,
        BellhopError::MalformedDebFilename { .. } => ExitCode::DataErr,
        BellhopError::AptlyCommandFailed { .. } => ExitCode::Software,
        BellhopError::AptlyNonZeroExit { .. } => ExitCode::Software,
        BellhopError::IoError(_) => ExitCode::Software,
        BellhopError::ArchiveExtractionFailed(_) => ExitCode::Software,
        BellhopError::AptlyNotFound => ExitCode::Software,
        BellhopError::InvalidGitHubReleaseUrl { .. } => ExitCode::DataErr,
        BellhopError::GitHubApiFailed { .. } => ExitCode::Software,
        BellhopError::NoAssetsInRelease { .. } => ExitCode::DataErr,
        BellhopError::DownloadFailed { .. } => ExitCode::Software,
        BellhopError::WatcherError(_) => ExitCode::Software,
    }
}
