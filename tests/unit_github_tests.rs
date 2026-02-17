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

use bellhop::gh;

#[test]
fn test_parse_valid_release_url() {
    let result =
        gh::parse_release_url("https://github.com/rabbitmq/rabbitmq-server/releases/tag/v4.2.3")
            .unwrap();
    assert_eq!(result.owner, "rabbitmq");
    assert_eq!(result.repo, "rabbitmq-server");
    assert_eq!(result.tag, "v4.2.3");
}

#[test]
fn test_parse_release_url_with_trailing_slash() {
    let result =
        gh::parse_release_url("https://github.com/rabbitmq/rabbitmq-server/releases/tag/v4.2.3/")
            .unwrap();
    assert_eq!(result.owner, "rabbitmq");
    assert_eq!(result.repo, "rabbitmq-server");
    assert_eq!(result.tag, "v4.2.3");
}

#[test]
fn test_parse_release_url_different_owner() {
    let result = gh::parse_release_url(
        "https://github.com/michaelklishin/rabbitmq-lqt/releases/tag/v0.20.0",
    )
    .unwrap();
    assert_eq!(result.owner, "michaelklishin");
    assert_eq!(result.repo, "rabbitmq-lqt");
    assert_eq!(result.tag, "v0.20.0");
}

#[test]
fn test_parse_release_url_complex_tag() {
    let result =
        gh::parse_release_url("https://github.com/owner/repo/releases/tag/my-release-v1.0.0-rc.1")
            .unwrap();
    assert_eq!(result.tag, "my-release-v1.0.0-rc.1");
}

#[test]
fn test_parse_release_url_without_tag_segment() {
    let result =
        gh::parse_release_url("https://github.com/rabbitmq/rabbitmq-server/releases/v4.2.4")
            .unwrap();
    assert_eq!(result.owner, "rabbitmq");
    assert_eq!(result.repo, "rabbitmq-server");
    assert_eq!(result.tag, "v4.2.4");
}

#[test]
fn test_parse_release_url_without_tag_segment_trailing_slash() {
    let result =
        gh::parse_release_url("https://github.com/rabbitmq/rabbitmq-server/releases/v4.2.4/")
            .unwrap();
    assert_eq!(result.owner, "rabbitmq");
    assert_eq!(result.repo, "rabbitmq-server");
    assert_eq!(result.tag, "v4.2.4");
}

#[test]
fn test_parse_invalid_url_not_github() {
    assert!(gh::parse_release_url("https://gitlab.com/owner/repo/releases/tag/v1.0").is_err());
}

#[test]
fn test_parse_invalid_url_not_release() {
    assert!(gh::parse_release_url("https://github.com/owner/repo/tree/main").is_err());
}

#[test]
fn test_parse_invalid_url_missing_tag() {
    assert!(gh::parse_release_url("https://github.com/owner/repo/releases/tag/").is_err());
}

#[test]
fn test_parse_invalid_url_releases_with_no_tag() {
    assert!(gh::parse_release_url("https://github.com/owner/repo/releases/").is_err());
}

#[test]
fn test_parse_invalid_url_too_short() {
    assert!(gh::parse_release_url("https://github.com/owner/repo").is_err());
}

#[test]
fn test_parse_invalid_url_empty_owner() {
    assert!(gh::parse_release_url("https://github.com//repo/releases/tag/v1.0").is_err());
}

#[test]
fn test_parse_invalid_url_empty_repo() {
    assert!(gh::parse_release_url("https://github.com/owner//releases/tag/v1.0").is_err());
}

#[test]
fn test_parse_invalid_url_random_string() {
    assert!(gh::parse_release_url("not-a-url-at-all").is_err());
}

#[test]
fn test_parse_http_url() {
    let result =
        gh::parse_release_url("http://github.com/rabbitmq/rabbitmq-server/releases/tag/v4.2.3")
            .unwrap();
    assert_eq!(result.owner, "rabbitmq");
    assert_eq!(result.repo, "rabbitmq-server");
    assert_eq!(result.tag, "v4.2.3");
}

#[test]
fn test_parse_release_url_with_whitespace() {
    let result =
        gh::parse_release_url("  https://github.com/owner/repo/releases/tag/v1.0  ").unwrap();
    assert_eq!(result.owner, "owner");
    assert_eq!(result.tag, "v1.0");
}
