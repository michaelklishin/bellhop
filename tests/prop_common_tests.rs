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
use proptest::prelude::*;

fn project_strategy() -> impl Strategy<Value = Project> {
    prop_oneof![
        Just(Project::RabbitMQ),
        Just(Project::Erlang),
        Just(Project::CliTools),
    ]
}

proptest! {
    #[test]
    fn project_display_never_empty(project in project_strategy()) {
        let s = project.to_string();
        prop_assert!(!s.is_empty());
        prop_assert!(s.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
    }
}
