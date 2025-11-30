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
#![allow(dead_code)]

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DebianFamily {
    Debian,
    Ubuntu,
}

impl FromStr for DebianFamily {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debian" => Ok(DebianFamily::Debian),
            "ubuntu" => Ok(DebianFamily::Ubuntu),
            _ => Err(format!("Unsupported Debian family: {s}")),
        }
    }
}

impl Display for DebianFamily {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DebianFamily::Debian => write!(f, "debian"),
            DebianFamily::Ubuntu => write!(f, "ubuntu"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DebianRelease {
    Trixie,
    Bookworm,
    Bullseye,
}

impl FromStr for DebianRelease {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trixie" => Ok(DebianRelease::Trixie),
            "bookworm" => Ok(DebianRelease::Bookworm),
            "bullseye" => Ok(DebianRelease::Bullseye),
            _ => Err(format!("Unsupported Debian release: {s}")),
        }
    }
}

impl Display for DebianRelease {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DebianRelease::Trixie => write!(f, "trixie"),
            DebianRelease::Bookworm => write!(f, "bookworm"),
            DebianRelease::Bullseye => write!(f, "bullseye"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UbuntuRelease {
    Noble,
    Jammy,
    Focal,
}

impl FromStr for UbuntuRelease {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "noble" => Ok(UbuntuRelease::Noble),
            "jammy" => Ok(UbuntuRelease::Jammy),
            "focal" => Ok(UbuntuRelease::Focal),
            _ => Err(format!("Unsupported Ubuntu release: {s}")),
        }
    }
}

impl Display for UbuntuRelease {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UbuntuRelease::Noble => write!(f, "noble"),
            UbuntuRelease::Jammy => write!(f, "jammy"),
            UbuntuRelease::Focal => write!(f, "focal"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Release {
    Debian(DebianRelease),
    Ubuntu(UbuntuRelease),
}

/// Reserved for future use
impl FromStr for Release {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(
                "Invalid release format, example to use: debian/bookworm or ubuntu/jammy"
                    .to_string(),
            );
        }

        let family = parts[0].parse::<DebianFamily>()?;

        match family {
            DebianFamily::Debian => {
                let release = parts[1]
                    .parse::<DebianRelease>()
                    .map_err(|_| format!("Invalid Debian release: {}", parts[1]))?;
                Ok(Release::Debian(release))
            }
            DebianFamily::Ubuntu => {
                let release = parts[1]
                    .parse::<UbuntuRelease>()
                    .map_err(|_| format!("Invalid Ubuntu release: {}", parts[1]))?;
                Ok(Release::Ubuntu(release))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DistributionAlias {
    Noble,
    Jammy,
    Focal,
    Trixie,
    Bookworm,
    Bullseye,
}

impl FromStr for DistributionAlias {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "noble" => Ok(DistributionAlias::Noble),
            "jammy" => Ok(DistributionAlias::Jammy),
            "focal" => Ok(DistributionAlias::Focal),
            "trixie" => Ok(DistributionAlias::Trixie),
            "bookworm" => Ok(DistributionAlias::Bookworm),
            "bullseye" => Ok(DistributionAlias::Bullseye),
            _ => Err(format!("Unsupported distribution alias: {s}")),
        }
    }
}

/// Reserved for future use
#[allow(dead_code)]
impl From<DistributionAlias> for Release {
    fn from(alias: DistributionAlias) -> Self {
        alias.to_release()
    }
}

impl Display for DistributionAlias {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DistributionAlias::Noble => write!(f, "noble"),
            DistributionAlias::Jammy => write!(f, "jammy"),
            DistributionAlias::Focal => write!(f, "focal"),
            DistributionAlias::Trixie => write!(f, "trixie"),
            DistributionAlias::Bookworm => write!(f, "bookworm"),
            DistributionAlias::Bullseye => write!(f, "bullseye"),
        }
    }
}

impl DistributionAlias {
    pub fn to_release(&self) -> Release {
        match self {
            DistributionAlias::Noble => Release::Ubuntu(UbuntuRelease::Noble),
            DistributionAlias::Jammy => Release::Ubuntu(UbuntuRelease::Jammy),
            DistributionAlias::Focal => Release::Ubuntu(UbuntuRelease::Focal),
            DistributionAlias::Trixie => Release::Debian(DebianRelease::Trixie),
            DistributionAlias::Bookworm => Release::Debian(DebianRelease::Bookworm),
            DistributionAlias::Bullseye => Release::Debian(DebianRelease::Bullseye),
        }
    }

    pub fn family(&self) -> DebianFamily {
        match self {
            DistributionAlias::Noble | DistributionAlias::Jammy | DistributionAlias::Focal => {
                DebianFamily::Ubuntu
            }
            DistributionAlias::Trixie
            | DistributionAlias::Bookworm
            | DistributionAlias::Bullseye => DebianFamily::Debian,
        }
    }

    pub fn family_name(&self) -> &'static str {
        match self {
            DistributionAlias::Noble | DistributionAlias::Jammy | DistributionAlias::Focal => {
                "ubuntu"
            }
            DistributionAlias::Trixie
            | DistributionAlias::Bookworm
            | DistributionAlias::Bullseye => "debian",
        }
    }

    pub fn release_name(&self) -> &'static str {
        match self {
            DistributionAlias::Noble => "noble",
            DistributionAlias::Jammy => "jammy",
            DistributionAlias::Focal => "focal",
            DistributionAlias::Trixie => "trixie",
            DistributionAlias::Bookworm => "bookworm",
            DistributionAlias::Bullseye => "bullseye",
        }
    }

    pub fn all() -> &'static [DistributionAlias] {
        const ALL_DISTRIBUTIONS: [DistributionAlias; 6] = [
            DistributionAlias::Noble,
            DistributionAlias::Jammy,
            DistributionAlias::Focal,
            DistributionAlias::Trixie,
            DistributionAlias::Bookworm,
            DistributionAlias::Bullseye,
        ];
        &ALL_DISTRIBUTIONS
    }

    pub fn erlang_supported() -> &'static [DistributionAlias] {
        const ERLANG_SUPPORTED: [DistributionAlias; 4] = [
            DistributionAlias::Noble,
            DistributionAlias::Jammy,
            DistributionAlias::Trixie,
            DistributionAlias::Bookworm,
        ];
        &ERLANG_SUPPORTED
    }
}
