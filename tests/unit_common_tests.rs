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

#[test]
fn test_project_display() {
    assert_eq!(Project::RabbitMQ.to_string(), "rabbitmq");
    assert_eq!(Project::Erlang.to_string(), "erlang");
}

#[test]
fn test_project_copy_clone() {
    let p1 = Project::RabbitMQ;
    let p2 = p1;
    assert_eq!(p1, p2);

    let p3 = p1.clone();
    assert_eq!(p1, p3);
}
