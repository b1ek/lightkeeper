use std::str::FromStr;

use strum_macros::{ EnumString, Display };
use serde_derive::{ Serialize, Deserialize };

use crate::utils::VersionNumber;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system, i.e. Windows, Linux...
    pub os: OperatingSystem,

    /// Numeric version.
    pub os_version: VersionNumber,

    /// Flavor covers different Windows OSes and Linux distributions.
    pub os_flavor: Flavor,
}

impl PlatformInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_unset(&self) -> bool {
        self.os == OperatingSystem::Unknown
    }

    // Version is given as str for convenience.
    pub fn is_newer_than(&self, flavor: Flavor, version: &str) -> bool {
        let parsed_version = VersionNumber::from_str(version).unwrap();

        self.os_flavor == flavor &&
        self.os_version > parsed_version
    }
}

#[derive(Clone, PartialEq, Eq, EnumString, Display, Serialize, Deserialize)]
pub enum OperatingSystem {
    Unknown,
    Windows,
    Linux,
}

impl Default for OperatingSystem {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, PartialEq, Eq, EnumString, Display, Serialize, Deserialize)]
pub enum Flavor {
    Unknown,

    // Windows:
    Server2012,
    Windows7,
    Windows10,
    Windows11,

    // Linux:
    Debian,
    Ubuntu,
    ArchLinux,
    RHEL,
}

impl Default for Flavor {
    fn default() -> Self {
        Self::Unknown
    }
}