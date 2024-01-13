use regex::Regex;
use std::{error, fmt};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn parse(value: &str) -> Result<Version, VersionError> {
        let captures = Regex::new(r"([0-9]|[1-9][0-9]+)\.([0-9]|[1-9][0-9]+)\.([0-9]|[1-9][0-9]+)")
            .unwrap()
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

    pub fn inrement_major(&self) -> Self {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }

    pub fn inrement_minor(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    pub fn inrement_patch(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
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
        Version::parse(v).map_err(|err| E::custom(err.to_string()))
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
