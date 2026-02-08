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
use crate::aptly;
use crate::common::Project;
use crate::deb::DistributionAlias;
use crate::errors::BellhopError;
use log::{debug, error, info, warn};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::fs;
use std::path::Path;
use std::sync::mpsc;

const RABBITMQ_SERVER_DIR: &str = "rabbitmq-server";
const RABBITMQ_ERLANG_DIR: &str = "rabbitmq-erlang";
const RABBITMQ_CLI_DIR: &str = "rabbitmq-cli";

pub fn project_for_directory(dir_name: &str) -> Option<Project> {
    match dir_name {
        RABBITMQ_SERVER_DIR => Some(Project::RabbitMQ),
        RABBITMQ_ERLANG_DIR => Some(Project::Erlang),
        RABBITMQ_CLI_DIR => Some(Project::CliTools),
        _ => None,
    }
}

fn subdirectories() -> [&'static str; 3] {
    [RABBITMQ_SERVER_DIR, RABBITMQ_ERLANG_DIR, RABBITMQ_CLI_DIR]
}

pub fn watch_directory(
    root: &Path,
    target_releases: &[DistributionAlias],
    max_events: Option<usize>,
) -> Result<(), BellhopError> {
    for subdir in subdirectories() {
        let dir_path = root.join(subdir);
        if !dir_path.exists() {
            info!("Creating watched directory: {}", dir_path.display());
            fs::create_dir_all(&dir_path)?;
        }
    }

    info!("Watching {} for .deb files", root.display());
    info!("Targeting {} distributions", target_releases.len());

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher =
        notify::recommended_watcher(tx).map_err(|e| BellhopError::WatcherError(e.to_string()))?;

    for subdir in subdirectories() {
        let dir_path = root.join(subdir);
        watcher
            .watch(&dir_path, RecursiveMode::NonRecursive)
            .map_err(|e| BellhopError::WatcherError(e.to_string()))?;
        info!("Watching: {}", dir_path.display());
    }

    let mut events_processed = 0;

    if max_events == Some(0) {
        return Ok(());
    }

    for event_result in rx {
        match event_result {
            Ok(event) => {
                debug!("Filesystem event: {event:?}");

                if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    continue;
                }

                for path in &event.paths {
                    if let Some(handled) = handle_file_event(path, target_releases) {
                        if handled {
                            events_processed += 1;
                        }
                    }
                }

                if let Some(max) = max_events {
                    if events_processed >= max {
                        info!("Reached max events ({max}), stopping watcher");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                error!("Watcher error: {e}");
            }
        }
    }

    Ok(())
}

pub fn releases_for_project<'a>(
    project: &Project,
    target_releases: &'a [DistributionAlias],
) -> Vec<&'a DistributionAlias> {
    let supported: &[DistributionAlias] = match project {
        Project::Erlang => DistributionAlias::erlang_supported(),
        Project::RabbitMQ | Project::CliTools => DistributionAlias::all(),
    };
    target_releases
        .iter()
        .filter(|d| supported.contains(d))
        .collect()
}

fn handle_file_event(path: &Path, target_releases: &[DistributionAlias]) -> Option<bool> {
    if !path.is_file() {
        return None;
    }

    let extension = path.extension()?.to_str()?;
    if extension != "deb" {
        warn!("Ignoring non-.deb file: {}", path.display());
        return Some(false);
    }

    let parent = path.parent()?;
    let dir_name = parent.file_name()?.to_str()?;

    let project = match project_for_directory(dir_name) {
        Some(p) => p,
        None => {
            warn!(
                "Ignoring file in unknown subdirectory '{}': {}",
                dir_name,
                path.display()
            );
            return Some(false);
        }
    };

    let applicable: Vec<DistributionAlias> = releases_for_project(&project, target_releases)
        .into_iter()
        .cloned()
        .collect();

    let filename = path.file_name()?.to_str()?;
    info!(
        "Importing {} into {} for {} distributions",
        filename,
        project,
        applicable.len()
    );

    match aptly::add_single_package_no_snapshot(&project, path, &applicable) {
        Ok(()) => {
            info!("Successfully imported {filename}");
            Some(true)
        }
        Err(e) => {
            error!("Failed to import {filename}: {e}");
            Some(false)
        }
    }
}
