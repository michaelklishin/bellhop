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

use bellhop::gh::releases::{ReleaseAsset, filter_assets, glob_match};

#[test]
fn test_glob_match_star_deb() {
    assert!(glob_match("*.deb", "rabbitmq-server_4.2.3-1_all.deb"));
    assert!(glob_match("*.deb", "rabbitmqadmin_2.25.0_amd64.deb"));
    assert!(!glob_match("*.deb", "rabbitmq-server.tar.gz"));
}

#[test]
fn test_glob_match_amd64_deb() {
    assert!(glob_match("*amd64*.deb", "rabbitmqadmin_2.25.0_amd64.deb"));
    assert!(!glob_match("*amd64*.deb", "rabbitmqadmin_2.25.0_arm64.deb"));
    assert!(!glob_match(
        "*amd64*.deb",
        "rabbitmq-server_4.2.3-1_all.deb"
    ));
}

#[test]
fn test_glob_match_exact() {
    assert!(glob_match("foo.deb", "foo.deb"));
    assert!(!glob_match("foo.deb", "bar.deb"));
}

#[test]
fn test_glob_match_leading_literal() {
    assert!(glob_match(
        "rabbitmq*.deb",
        "rabbitmq-server_4.2.3-1_all.deb"
    ));
    assert!(!glob_match("rabbitmq*.deb", "erlang_26.0_amd64.deb"));
}

#[test]
fn test_glob_match_star_only() {
    assert!(glob_match("*", "anything.deb"));
    assert!(glob_match("*", ""));
}

#[test]
fn test_glob_match_consecutive_stars() {
    assert!(glob_match("**.deb", "test.deb"));
    assert!(glob_match("test**.deb", "test.deb"));
    assert!(glob_match("test**.deb", "test-foo.deb"));
}

fn make_asset(name: &str) -> ReleaseAsset {
    ReleaseAsset {
        name: name.to_string(),
        browser_download_url: format!("https://example.com/{name}"),
        size: 100,
    }
}

#[test]
fn test_filter_assets_star_deb() {
    let assets = vec![
        make_asset("rabbitmq-server_4.2.3-1_all.deb"),
        make_asset("rabbitmq-server-4.2.3.tar.xz"),
        make_asset("rabbitmq-server-4.2.3.exe"),
    ];
    let filtered = filter_assets(assets, "*.deb");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "rabbitmq-server_4.2.3-1_all.deb");
}

#[test]
fn test_filter_assets_amd64_deb() {
    let assets = vec![
        make_asset("rabbitmqadmin_2.25.0_amd64.deb"),
        make_asset("rabbitmqadmin_2.25.0_arm64.deb"),
        make_asset("rabbitmqadmin-2.25.0.tar.gz"),
    ];
    let filtered = filter_assets(assets, "*amd64*.deb");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "rabbitmqadmin_2.25.0_amd64.deb");
}

#[test]
fn test_filter_assets_no_matches() {
    let assets = vec![make_asset("README.md"), make_asset("source.tar.gz")];
    let filtered = filter_assets(assets, "*.deb");
    assert!(filtered.is_empty());
}
