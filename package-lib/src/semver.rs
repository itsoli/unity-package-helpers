use std::error;
use std::fmt;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

/// Error parsing a semantic version.
#[derive(Debug)]
pub enum VersionError {
    InvalidVersionString,
    MajorOutOfRange,
    MinorOutOfRange,
    PatchOutOfRange,
}

impl error::Error for VersionError {}

impl fmt::Display for VersionError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::VersionError::*;
        match self {
            InvalidVersionString => write!(fmt, "Invalid version string"),
            MajorOutOfRange => write!(fmt, "Major version out of u16 range"),
            MinorOutOfRange => write!(fmt, "Minor version out of u16 range"),
            PatchOutOfRange => write!(fmt, "Patch version out of u16 range"),
        }
    }
}

/// Semantic version.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    /// Returns the next version with the major number incremented.
    pub fn inrement_major(&self) -> Self {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }

    /// Returns the next version with the minor number incremented.
    pub fn inrement_minor(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    /// Returns the next version with the patch number incremented.
    pub fn inrement_patch(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
    }
}

impl FromStr for Version {
    type Err = VersionError;

    /// Parses a semantic version from a string.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref VERSION_PATTERN: Regex =
                Regex::new(r"([0-9]|[1-9][0-9]+)\.([0-9]|[1-9][0-9]+)\.([0-9]|[1-9][0-9]+)")
                    .unwrap();
        }
        let captures = VERSION_PATTERN
            .captures(value)
            .ok_or(VersionError::InvalidVersionString)?;
        Ok(Version {
            major: captures[1]
                .parse::<u16>()
                .map_err(|_| VersionError::MajorOutOfRange)?,
            minor: captures[2]
                .parse::<u16>()
                .map_err(|_| VersionError::MinorOutOfRange)?,
            patch: captures[3]
                .parse::<u16>()
                .map_err(|_| VersionError::PatchOutOfRange)?,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string in the format \"major_number.minor_number.patch_number\"")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<Version>()
            .map_err(|err| E::custom(err.to_string()))
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}
