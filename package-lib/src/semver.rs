use std::error;
use std::fmt;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

/// Error parsing a semantic version.
#[derive(Debug)]
pub enum VersionError {
    Empty,
    UnexpectedEnd(Position),
    UnexpectedChar(Position, char),
    UnexpectedCharAfter(Position, char),
    LeadingZero(Position),
    Overflow(Position),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Position {
    Major,
    Minor,
    Patch,
}

impl error::Error for VersionError {}

impl fmt::Display for VersionError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::VersionError::*;
        match self {
            Empty => write!(formatter, "Empty version string"),
            UnexpectedEnd(pos) => {
                write!(formatter, "Unexpected end of input while parsing {}", pos)
            }
            UnexpectedChar(pos, ch) => write!(
                formatter,
                "Unexpected character '{}' while parsing {}",
                *ch, pos
            ),
            UnexpectedCharAfter(pos, ch) => {
                write!(formatter, "Unexpected character '{}' after {}", *ch, pos)
            }
            LeadingZero(pos) => write!(formatter, "Leading zero while parsing {}", pos),
            Overflow(pos) => write!(formatter, "Value of {} exceeds u16::MAX", pos),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::Position::*;
        formatter.write_str(match self {
            Major => "major version number",
            Minor => "minor version number",
            Patch => "patch version number",
        })
    }
}

/// Semantic version. Does not support pre-release or build metadata.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    /// Returns the next version with the major number incremented.
    pub fn increment_major(&self) -> Self {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }

    /// Returns the next version with the minor number incremented.
    pub fn increment_minor(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    /// Returns the next version with the patch number incremented.
    pub fn increment_patch(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    /// Parses a semantic version from a string.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err(VersionError::Empty);
        }

        let mut pos = Position::Major;
        let (major, value) = parse_number(value, pos)?;
        let value = parse_dot(value, pos)?;

        pos = Position::Minor;
        let (minor, value) = parse_number(value, pos)?;
        let value = parse_dot(value, pos)?;

        pos = Position::Patch;
        let (patch, value) = parse_number(value, pos)?;

        if let Some(unexpected) = value.chars().next() {
            return Err(VersionError::UnexpectedCharAfter(pos, unexpected));
        }

        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

fn parse_number(input: &str, pos: Position) -> Result<(u16, &str), VersionError> {
    let mut len = 0;
    let mut value = 0u16;

    while let Some(&digit) = input.as_bytes().get(len) {
        if !digit.is_ascii_digit() {
            break;
        }
        if value == 0 && len > 0 {
            return Err(VersionError::LeadingZero(pos));
        }
        match value
            .checked_mul(10)
            .and_then(|value| value.checked_add((digit - b'0') as u16))
        {
            Some(sum) => value = sum,
            None => return Err(VersionError::Overflow(pos)),
        }
        len += 1;
    }

    if len > 0 {
        Ok((value, &input[len..]))
    } else if let Some(unexpected) = input[len..].chars().next() {
        Err(VersionError::UnexpectedChar(pos, unexpected))
    } else {
        Err(VersionError::UnexpectedEnd(pos))
    }
}

fn parse_dot(input: &str, pos: Position) -> Result<&str, VersionError> {
    if let Some(rest) = input.strip_prefix('.') {
        Ok(rest)
    } else if let Some(unexpected) = input.chars().next() {
        Err(VersionError::UnexpectedChar(pos, unexpected))
    } else {
        Err(VersionError::UnexpectedEnd(pos))
    }
}

struct VersionVisitor;

impl Visitor<'_> for VersionVisitor {
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
