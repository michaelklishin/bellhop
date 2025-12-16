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

#[test]
fn test_debian_family_display() {
    assert_eq!(DebianFamily::Debian.to_string(), "debian");
    assert_eq!(DebianFamily::Ubuntu.to_string(), "ubuntu");
}

#[test]
fn test_debian_family_from_str() {
    assert_eq!(
        "debian".parse::<DebianFamily>().unwrap(),
        DebianFamily::Debian
    );
    assert_eq!(
        "ubuntu".parse::<DebianFamily>().unwrap(),
        DebianFamily::Ubuntu
    );
    assert!("redhat".parse::<DebianFamily>().is_err());
}

#[test]
fn test_debian_family_roundtrip() {
    for family in [DebianFamily::Debian, DebianFamily::Ubuntu] {
        let s = family.to_string();
        let parsed: DebianFamily = s.parse().unwrap();
        assert_eq!(family, parsed);
    }
}

#[test]
fn test_debian_release_display() {
    assert_eq!(DebianRelease::Trixie.to_string(), "trixie");
    assert_eq!(DebianRelease::Bookworm.to_string(), "bookworm");
    assert_eq!(DebianRelease::Bullseye.to_string(), "bullseye");
}

#[test]
fn test_debian_release_from_str() {
    assert_eq!(
        "trixie".parse::<DebianRelease>().unwrap(),
        DebianRelease::Trixie
    );
    assert_eq!(
        "bookworm".parse::<DebianRelease>().unwrap(),
        DebianRelease::Bookworm
    );
    assert_eq!(
        "bullseye".parse::<DebianRelease>().unwrap(),
        DebianRelease::Bullseye
    );
    assert!("jessie".parse::<DebianRelease>().is_err());
}

#[test]
fn test_debian_release_roundtrip() {
    for release in [
        DebianRelease::Trixie,
        DebianRelease::Bookworm,
        DebianRelease::Bullseye,
    ] {
        let s = release.to_string();
        let parsed: DebianRelease = s.parse().unwrap();
        assert_eq!(release, parsed);
    }
}

#[test]
fn test_ubuntu_release_display() {
    assert_eq!(UbuntuRelease::Noble.to_string(), "noble");
    assert_eq!(UbuntuRelease::Jammy.to_string(), "jammy");
    assert_eq!(UbuntuRelease::Focal.to_string(), "focal");
}

#[test]
fn test_ubuntu_release_from_str() {
    assert_eq!(
        "noble".parse::<UbuntuRelease>().unwrap(),
        UbuntuRelease::Noble
    );
    assert_eq!(
        "jammy".parse::<UbuntuRelease>().unwrap(),
        UbuntuRelease::Jammy
    );
    assert_eq!(
        "focal".parse::<UbuntuRelease>().unwrap(),
        UbuntuRelease::Focal
    );
    assert!("bionic".parse::<UbuntuRelease>().is_err());
}

#[test]
fn test_ubuntu_release_roundtrip() {
    for release in [
        UbuntuRelease::Noble,
        UbuntuRelease::Jammy,
        UbuntuRelease::Focal,
    ] {
        let s = release.to_string();
        let parsed: UbuntuRelease = s.parse().unwrap();
        assert_eq!(release, parsed);
    }
}

#[test]
fn test_distribution_alias_display() {
    assert_eq!(DistributionAlias::Noble.to_string(), "noble");
    assert_eq!(DistributionAlias::Bookworm.to_string(), "bookworm");
}

#[test]
fn test_distribution_alias_from_str() {
    assert_eq!(
        "noble".parse::<DistributionAlias>().unwrap(),
        DistributionAlias::Noble
    );
    assert_eq!(
        "bookworm".parse::<DistributionAlias>().unwrap(),
        DistributionAlias::Bookworm
    );
    assert!("invalid".parse::<DistributionAlias>().is_err());
}

#[test]
fn test_distribution_alias_family() {
    assert_eq!(DistributionAlias::Bookworm.family(), DebianFamily::Debian);
    assert_eq!(DistributionAlias::Jammy.family(), DebianFamily::Ubuntu);
}

#[test]
fn test_distribution_alias_family_name() {
    assert_eq!(DistributionAlias::Bookworm.family_name(), "debian");
    assert_eq!(DistributionAlias::Jammy.family_name(), "ubuntu");
}

#[test]
fn test_distribution_alias_release_name() {
    assert_eq!(DistributionAlias::Bookworm.release_name(), "bookworm");
    assert_eq!(DistributionAlias::Noble.release_name(), "noble");
}

#[test]
fn test_distribution_alias_all() {
    let all = DistributionAlias::all();
    assert_eq!(all.len(), 6);
    assert!(all.contains(&DistributionAlias::Bookworm));
    assert!(all.contains(&DistributionAlias::Noble));
}

#[test]
fn test_distribution_alias_erlang_supported() {
    let supported = DistributionAlias::erlang_supported();
    assert_eq!(supported.len(), 4);
    assert!(supported.contains(&DistributionAlias::Bookworm));
    assert!(supported.contains(&DistributionAlias::Noble));
    assert!(!supported.contains(&DistributionAlias::Focal));
    assert!(!supported.contains(&DistributionAlias::Bullseye));
}

#[test]
fn test_distribution_alias_to_release() {
    assert_eq!(
        DistributionAlias::Bookworm.to_release(),
        Release::Debian(DebianRelease::Bookworm)
    );
    assert_eq!(
        DistributionAlias::Jammy.to_release(),
        Release::Ubuntu(UbuntuRelease::Jammy)
    );
}

#[test]
fn test_release_from_str_valid() {
    assert_eq!(
        "debian/bookworm".parse::<Release>().unwrap(),
        Release::Debian(DebianRelease::Bookworm)
    );
    assert_eq!(
        "ubuntu/jammy".parse::<Release>().unwrap(),
        Release::Ubuntu(UbuntuRelease::Jammy)
    );
}

#[test]
fn test_release_from_str_invalid_format() {
    assert!("bookworm".parse::<Release>().is_err());
    assert!("debian/bookworm/extra".parse::<Release>().is_err());
}

#[test]
fn test_release_from_str_invalid_family() {
    assert!("redhat/9".parse::<Release>().is_err());
}

#[test]
fn test_release_from_str_invalid_release() {
    assert!("debian/jessie".parse::<Release>().is_err());
    assert!("ubuntu/bionic".parse::<Release>().is_err());
}
