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
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Project {
    RabbitMQ,
    Erlang,
    CliTools,
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Project::RabbitMQ => write!(f, "rabbitmq"),
            Project::Erlang => write!(f, "erlang"),
            Project::CliTools => write!(f, "cli-tools"),
        }
    }
}
