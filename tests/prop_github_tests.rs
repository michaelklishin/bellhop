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
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_urls_always_parse(
        owner in "[a-zA-Z0-9_-]{1,20}",
        repo in "[a-zA-Z0-9_.-]{1,30}",
        tag in "[a-zA-Z0-9._-]{1,20}"
    ) {
        let url = format!("https://github.com/{owner}/{repo}/releases/tag/{tag}");
        let result = gh::parse_release_url(&url).unwrap();
        prop_assert_eq!(result.owner, owner);
        prop_assert_eq!(result.repo, repo);
        prop_assert_eq!(result.tag, tag);
    }

    #[test]
    fn random_strings_never_panic(s in "\\PC{0,200}") {
        let _ = gh::parse_release_url(&s);
    }
}
