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

use bellhop::deb::{DebianFamily, DebianRelease, DistributionAlias, Release, UbuntuRelease};
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

fn debian_family_strategy() -> impl Strategy<Value = DebianFamily> {
    prop_oneof![Just(DebianFamily::Debian), Just(DebianFamily::Ubuntu),]
}

fn debian_release_strategy() -> impl Strategy<Value = DebianRelease> {
    prop_oneof![
        Just(DebianRelease::Trixie),
        Just(DebianRelease::Bookworm),
        Just(DebianRelease::Bullseye),
    ]
}

fn ubuntu_release_strategy() -> impl Strategy<Value = UbuntuRelease> {
    prop_oneof![
        Just(UbuntuRelease::Noble),
        Just(UbuntuRelease::Jammy),
        Just(UbuntuRelease::Focal),
    ]
}

proptest! {
    #[test]
    fn distribution_alias_roundtrip(alias in distribution_alias_strategy()) {
        let s = alias.to_string();
        let parsed: DistributionAlias = s.parse().unwrap();
        prop_assert_eq!(alias, parsed);
    }

    #[test]
    fn debian_family_roundtrip(family in debian_family_strategy()) {
        let s = family.to_string();
        let parsed: DebianFamily = s.parse().unwrap();
        prop_assert_eq!(family, parsed);
    }

    #[test]
    fn debian_release_roundtrip(release in debian_release_strategy()) {
        let s = release.to_string();
        let parsed: DebianRelease = s.parse().unwrap();
        prop_assert_eq!(release, parsed);
    }

    #[test]
    fn ubuntu_release_roundtrip(release in ubuntu_release_strategy()) {
        let s = release.to_string();
        let parsed: UbuntuRelease = s.parse().unwrap();
        prop_assert_eq!(release, parsed);
    }

    #[test]
    fn distribution_alias_family_matches_release(alias in distribution_alias_strategy()) {
        let release = alias.to_release();
        match (&alias, &release) {
            (DistributionAlias::Noble | DistributionAlias::Jammy | DistributionAlias::Focal, Release::Ubuntu(_)) => {},
            (DistributionAlias::Trixie | DistributionAlias::Bookworm | DistributionAlias::Bullseye, Release::Debian(_)) => {},
            _ => prop_assert!(false, "Family mismatch for {:?}", alias),
        }
    }
}
