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
pub mod downloads;
pub mod releases;

use crate::errors::BellhopError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRelease {
    pub owner: String,
    pub repo: String,
    pub tag: String,
}

pub fn parse_release_url(url: &str) -> Result<GitHubRelease, BellhopError> {
    let url = url.trim().trim_end_matches('/');

    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .ok_or_else(|| BellhopError::InvalidGitHubReleaseUrl {
            url: url.to_string(),
        })?;

    // Expected format: {owner}/{repo}/releases/tag/{tag}
    let parts: Vec<&str> = path.splitn(5, '/').collect();
    if parts.len() != 5 || parts[2] != "releases" || parts[3] != "tag" {
        return Err(BellhopError::InvalidGitHubReleaseUrl {
            url: url.to_string(),
        });
    }

    let owner = parts[0];
    let repo = parts[1];
    let tag = parts[4];

    if owner.is_empty() || repo.is_empty() || tag.is_empty() {
        return Err(BellhopError::InvalidGitHubReleaseUrl {
            url: url.to_string(),
        });
    }

    Ok(GitHubRelease {
        owner: owner.to_string(),
        repo: repo.to_string(),
        tag: tag.to_string(),
    })
}
