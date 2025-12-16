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

use bellhop::archive::extract_version_from_filename;
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_deb_filenames_parse_successfully(
        name in "[a-z][a-z0-9-]{2,20}",
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
        revision in 0u32..10,
        arch in prop_oneof!["amd64", "arm64", "armel", "armhf", "i386", "all"]
    ) {
        let version_str = format!("{major}.{minor}.{patch}-{revision}");
        let filename = format!("{name}_{version_str}_{arch}.deb");
        let result = extract_version_from_filename(&filename);
        prop_assert!(result.is_ok(), "Failed to parse: {}", filename);
        prop_assert_eq!(result.unwrap(), version_str);
    }

    #[test]
    fn filenames_without_deb_extension_fail(s in "[a-z0-9_.-]+") {
        if !s.ends_with(".deb") {
            let result = extract_version_from_filename(&s);
            prop_assert!(result.is_err());
        }
    }

    #[test]
    fn epoch_versions_parse_correctly(
        name in "[a-z][a-z0-9-]{2,10}",
        epoch in 0u32..10,
        version in "[0-9.]+",
        revision in 0u32..10,
        arch in prop_oneof!["amd64", "all"]
    ) {
        let version_str = format!("{epoch}:{version}-{revision}");
        let filename = format!("{name}_{version_str}_{arch}.deb");
        match extract_version_from_filename(&filename) {
            Ok(v) => prop_assert_eq!(v, version_str),
            Err(_) => {}
        }
    }
}
