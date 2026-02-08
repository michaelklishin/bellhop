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
use crate::gh::GitHubRelease;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    assets: Vec<ReleaseAsset>,
}

pub fn fetch_release_assets(
    client: &Client,
    release: &GitHubRelease,
) -> Result<Vec<ReleaseAsset>, BellhopError> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        release.owner, release.repo, release.tag
    );

    let response = client
        .get(&api_url)
        .header("User-Agent", "bellhop")
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| BellhopError::GitHubApiFailed {
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(BellhopError::GitHubApiFailed {
            message: format!(
                "GitHub API returned status {} for {}",
                response.status(),
                api_url
            ),
        });
    }

    let release_data: ReleaseResponse =
        response.json().map_err(|e| BellhopError::GitHubApiFailed {
            message: format!("Failed to parse GitHub API response: {e}"),
        })?;

    Ok(release_data.assets)
}

pub fn filter_assets(assets: Vec<ReleaseAsset>, pattern: &str) -> Vec<ReleaseAsset> {
    assets
        .into_iter()
        .filter(|a| glob_match(pattern, &a.name))
        .collect()
}

pub fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        return name == pattern;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match name[pos..].find(part) {
            Some(idx) => {
                if i == 0 && idx != 0 {
                    return false;
                }
                pos += idx + part.len();
            }
            None => return false,
        }
    }

    match parts.last() {
        Some(suffix) if !suffix.is_empty() => name.ends_with(suffix),
        _ => true,
    }
}
