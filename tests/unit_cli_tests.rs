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

use bellhop::cli;
use chrono::Local;
use clap::ArgMatches;

// Drills down to the leaf subcommand's matches, e.g. rabbitmq -> deb -> publish.
fn leaf_matches(args: &[&str]) -> ArgMatches {
    let mut matches = cli::parser()
        .try_get_matches_from(args)
        .expect("arguments should parse");
    while let Some((_, sub)) = matches.remove_subcommand() {
        matches = sub;
    }
    matches
}

#[test]
fn test_publish_accepts_suffix() {
    let matches = leaf_matches(&[
        "bellhop",
        "rabbitmq",
        "deb",
        "publish",
        "-d",
        "bookworm",
        "--suffix",
        "07-Aug-25",
    ]);
    assert_eq!(cli::suffix(&matches), "07-Aug-25");
}

#[test]
fn test_publish_suffix_defaults_to_today() {
    let matches = leaf_matches(&["bellhop", "rabbitmq", "deb", "publish", "-d", "bookworm"]);
    let expected = Local::now().format("%d-%b-%y").to_string();
    assert_eq!(cli::suffix(&matches), expected);
}

#[test]
fn test_add_accepts_suffix() {
    let matches = leaf_matches(&[
        "bellhop", "rabbitmq", "deb", "add", "-p", "pkg.deb", "-d", "bookworm", "--suffix", "v2",
    ]);
    assert_eq!(cli::suffix(&matches), "v2");
}

#[test]
fn test_publish_still_requires_a_distribution() {
    let result = cli::parser().try_get_matches_from(["bellhop", "rabbitmq", "deb", "publish"]);
    assert!(
        result.is_err(),
        "publish must still require a distribution selector"
    );
}
