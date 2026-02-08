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
use crate::gh::releases::ReleaseAsset;
use log::info;
use reqwest::blocking::Client;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

pub fn download_assets(
    client: &Client,
    assets: &[ReleaseAsset],
    dest_dir: &Path,
) -> Result<Vec<PathBuf>, BellhopError> {
    let mut paths = Vec::with_capacity(assets.len());

    for asset in assets {
        let dest_path = dest_dir.join(&asset.name);
        info!("Downloading {} ({} bytes)", asset.name, asset.size);

        let mut response = client
            .get(&asset.browser_download_url)
            .header("User-Agent", "bellhop")
            .send()
            .map_err(|e| BellhopError::DownloadFailed {
                url: asset.browser_download_url.clone(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(BellhopError::DownloadFailed {
                url: asset.browser_download_url.clone(),
                message: format!("HTTP status {}", response.status()),
            });
        }

        let mut file = File::create(&dest_path)?;
        io::copy(&mut response, &mut file).map_err(|e| BellhopError::DownloadFailed {
            url: asset.browser_download_url.clone(),
            message: e.to_string(),
        })?;

        info!("Downloaded {}", asset.name);
        paths.push(dest_path);
    }

    Ok(paths)
}
