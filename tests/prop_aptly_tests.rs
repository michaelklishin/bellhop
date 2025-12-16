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
use proptest::prelude::*;

fn distribution_alias_strategy() -> impl Strategy<Value = DistributionAlias> {
    prop_oneof![
        Just(DistributionAlias::Noble),
        Just(DistributionAlias::Jammy),
        Just(DistributionAlias::Focal),
        Just(DistributionAlias::Trixie),
        Just(DistributionAlias::Bookworm),
        Just(DistributionAlias::Bullseye),
    ]
}

fn project_strategy() -> impl Strategy<Value = Project> {
    prop_oneof![Just(Project::RabbitMQ), Just(Project::Erlang),]
}

proptest! {
    #[test]
    fn repo_names_never_contain_invalid_path_chars(
        project in project_strategy(),
        dist in distribution_alias_strategy()
    ) {
        let name = bellhop::aptly::repo_name(&project, &dist);
        prop_assert!(!name.contains('/'));
        prop_assert!(!name.contains('\\'));
        prop_assert!(!name.contains('\0'));
    }

    #[test]
    fn repo_names_always_start_with_repo_prefix(
        project in project_strategy(),
        dist in distribution_alias_strategy()
    ) {
        let name = bellhop::aptly::repo_name(&project, &dist);
        prop_assert!(name.starts_with("repo-"));
    }

    #[test]
    fn snapshot_names_never_contain_invalid_chars(
        project in project_strategy(),
        dist in distribution_alias_strategy(),
        suffix in "[A-Za-z0-9-]+"
    ) {
        let name = bellhop::aptly::snapshot_name_with_suffix(&project, &dist, &suffix);
        prop_assert!(name.starts_with("snap-"));
        prop_assert!(!name.contains('/'));
        prop_assert!(!name.contains('\\'));
    }

    #[test]
    fn rel_paths_are_valid_posix_paths(
        project in project_strategy(),
        dist in distribution_alias_strategy()
    ) {
        let path = bellhop::aptly::rel_path_with_prefix(&project, &dist);
        let parts: Vec<&str> = path.split('/').collect();
        prop_assert_eq!(parts.len(), 3);
        prop_assert!(!path.starts_with('/'));
        prop_assert!(!path.ends_with('/'));
    }
}
