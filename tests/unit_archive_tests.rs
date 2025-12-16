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

use bellhop::archive::{extract_version_from_filename, extract_versions_from_debs};
use std::path::PathBuf;

#[test]
fn test_extract_version_standard_format() {
    assert_eq!(
        extract_version_from_filename("rabbitmq-server_4.1.3-1_all.deb").unwrap(),
        "4.1.3-1"
    );
}

#[test]
fn test_extract_version_with_epoch() {
    assert_eq!(
        extract_version_from_filename("erlang-base_1:27.3.4.6-1_amd64.deb").unwrap(),
        "1:27.3.4.6-1"
    );
}

#[test]
fn test_extract_version_complex_package_name() {
    assert_eq!(
        extract_version_from_filename("lib-foo-bar-baz_2.0.1-1_arm64.deb").unwrap(),
        "2.0.1-1"
    );
}

#[test]
fn test_extract_version_various_architectures() {
    for arch in ["amd64", "arm64", "armel", "armhf", "i386", "all"] {
        let filename = format!("package_1.2.3-1_{arch}.deb");
        assert_eq!(extract_version_from_filename(&filename).unwrap(), "1.2.3-1");
    }
}

#[test]
fn test_extract_version_not_deb_file() {
    let result = extract_version_from_filename("package_1.2.3-1_amd64.rpm");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not a .deb file"));
}

#[test]
fn test_extract_version_invalid_format_too_few_parts() {
    let result = extract_version_from_filename("invalid.deb");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Malformed .deb filename")
    );
}

#[test]
fn test_extract_version_missing_architecture() {
    let result = extract_version_from_filename("package_1.2.3-1.deb");
    assert!(result.is_err());
}

#[test]
fn test_extract_versions_from_multiple_debs() {
    let paths = vec![
        PathBuf::from("rabbitmq-server_4.1.3-1_all.deb"),
        PathBuf::from("rabbitmq-server_4.1.4-1_all.deb"),
    ];
    let versions = extract_versions_from_debs(&paths).unwrap();
    assert_eq!(versions, vec!["4.1.3-1", "4.1.4-1"]);
}
