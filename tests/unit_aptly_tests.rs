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

use bellhop::common::Project;
use bellhop::deb::DistributionAlias;

#[test]
fn test_repo_name_rabbitmq() {
    assert_eq!(
        bellhop::aptly::repo_name(&Project::RabbitMQ, &DistributionAlias::Bookworm),
        "repo-rabbitmq-server-bookworm"
    );
    assert_eq!(
        bellhop::aptly::repo_name(&Project::RabbitMQ, &DistributionAlias::Noble),
        "repo-rabbitmq-server-noble"
    );
}

#[test]
fn test_repo_name_erlang() {
    assert_eq!(
        bellhop::aptly::repo_name(&Project::Erlang, &DistributionAlias::Trixie),
        "repo-rabbitmq-erlang-trixie"
    );
    assert_eq!(
        bellhop::aptly::repo_name(&Project::Erlang, &DistributionAlias::Jammy),
        "repo-rabbitmq-erlang-jammy"
    );
}

#[test]
fn test_project_prefix() {
    assert_eq!(
        bellhop::aptly::project_prefix(&Project::RabbitMQ),
        "rabbitmq-server"
    );
    assert_eq!(
        bellhop::aptly::project_prefix(&Project::Erlang),
        "rabbitmq-erlang"
    );
}

#[test]
fn test_snapshot_name_with_suffix_rabbitmq() {
    let name = bellhop::aptly::snapshot_name_with_suffix(
        &Project::RabbitMQ,
        &DistributionAlias::Bookworm,
        "16-Dec-25",
    );
    assert_eq!(name, "snap-rabbitmq-server-bookworm-16-Dec-25");
}

#[test]
fn test_snapshot_name_with_suffix_erlang() {
    let name = bellhop::aptly::snapshot_name_with_suffix(
        &Project::Erlang,
        &DistributionAlias::Trixie,
        "16-Dec-25",
    );
    assert_eq!(name, "snap-rabbitmq-erlang-trixie-16-Dec-25");
}

#[test]
fn test_rel_path_with_prefix_debian() {
    assert_eq!(
        bellhop::aptly::rel_path_with_prefix(&Project::RabbitMQ, &DistributionAlias::Bookworm),
        "rabbitmq-server/debian/bookworm"
    );
}

#[test]
fn test_rel_path_with_prefix_ubuntu() {
    assert_eq!(
        bellhop::aptly::rel_path_with_prefix(&Project::Erlang, &DistributionAlias::Noble),
        "rabbitmq-erlang/ubuntu/noble"
    );
}

#[test]
fn test_all_distributions_have_valid_repo_names() {
    for dist in DistributionAlias::all() {
        for project in [Project::RabbitMQ, Project::Erlang] {
            let repo = bellhop::aptly::repo_name(&project, dist);
            assert!(repo.starts_with("repo-"));
            assert!(!repo.contains('/'));
            assert!(!repo.contains('\\'));
        }
    }
}
