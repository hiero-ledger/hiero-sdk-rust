// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::str::FromStr;

use hedera_proto::services;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::Error;

#[cfg(test)]
mod tests;

/// Hiero follows [semantic versioning](https://semver.org) for both the HAPI protobufs and
/// the Services software.
#[derive(Debug, Clone)]
pub struct SemanticVersion {
    /// Increases with incompatible API changes
    pub major: u32,

    /// Increases with backwards-compatible new functionality
    pub minor: u32,

    /// Increases with backwards-compatible bug fixes]
    pub patch: u32,

    /// A pre-release version MAY be denoted by appending a hyphen and a series of dot separated identifiers (<https://semver.org/#spec-item-9>);
    /// so given a semver 0.14.0-alpha.1+21AF26D3, this field would contain ‘alpha.1’
    pub prerelease: String,

    /// Build metadata MAY be denoted by appending a plus sign and a series of dot separated identifiers
    /// immediately following the patch or pre-release version (<https://semver.org/#spec-item-10>);
    /// so given a semver 0.14.0-alpha.1+21AF26D3, this field would contain ‘21AF26D3’
    pub build: String,
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.prerelease.is_empty() {
            write!(f, "-{}", &self.prerelease)?;
        }

        if !self.build.is_empty() {
            write!(f, "+{}", &self.build)?;
        }

        Ok(())
    }
}

fn is_valid_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}

/// Returns true if this is a numeric string, and it starts with `"0"`, and isn't exactly `"0"`
fn is_numeric_with_leading_zero(s: &str) -> bool {
    s.strip_prefix('0')
        .map_or(false, |rest| !rest.is_empty() && rest.chars().all(|it| it.is_ascii_digit()))
}

fn parse_version(s: &str, name: &str) -> crate::Result<u32> {
    if is_numeric_with_leading_zero(s) {
        return Err(Error::basic_parse(format!(
            "semver section `{name}` starts with leading 0: `{s}`"
        )));
    }

    s.parse().map_err(Error::basic_parse)
}

fn parse_prerelease(s: &str) -> crate::Result<String> {
    if s.is_empty() {
        return Err(Error::basic_parse("semver with empty rerelease"));
    }

    for identifier in s.split('.') {
        if identifier.is_empty() {
            return Err(Error::basic_parse("semver with empty -pre identifier"));
        }

        if !identifier.chars().all(is_valid_ident_char) {
            return Err(Error::basic_parse(
                "semver with invalid identifier for the -pre section: `{identifier}`",
            ));
        }

        if is_numeric_with_leading_zero(identifier) {
            return Err(Error::basic_parse(
                "numeric pre-release identifier has leading zero: `{identifier}`",
            ));
        }
    }

    Ok(s.to_owned())
}

fn parse_build(s: &str) -> crate::Result<String> {
    if s.is_empty() {
        return Err(Error::basic_parse("semver with empty build"));
    }

    for identifier in s.split('.') {
        if identifier.is_empty() {
            return Err(Error::basic_parse("semver with empty build-section identifier"));
        }

        if !identifier.chars().all(is_valid_ident_char) {
            return Err(Error::basic_parse(
                "semver with invalid identifier for the build section: `{identifier}",
            ));
        }
    }

    Ok(s.to_owned())
}

impl FromStr for SemanticVersion {
    type Err = Error;

    // its probably useless doing strict parsing when Hiero probably accepts loose parsing anyway, but lets at least *try* not to make it worse.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(3, '.').collect();

        match &*parts {
            [major, minor, rest] => {
                let (patch, pre, build) = {
                    // while it seems like this is a weird order,
                    // it actually makes more sense than `pre` first,
                    // because `pre` is in the middle of the string.
                    let (rest, build) = match rest.split_once('+') {
                        Some((rest, build)) => (rest, Some(build)),
                        None => (*rest, None),
                    };

                    let (patch, pre) = match rest.split_once('-') {
                        Some((patch, pre)) => (patch, Some(pre)),
                        None => (rest, None),
                    };

                    (patch, pre, build)
                };

                let major = parse_version(major, "major")?;
                let minor = parse_version(minor, "minor")?;
                let patch = parse_version(patch, "patch")?;

                let prerelease = match pre {
                    Some(it) => parse_prerelease(it)?,
                    None => String::new(),
                };

                let build = match build {
                    Some(it) => parse_build(it)?,
                    None => String::new(),
                };

                Ok(Self { major, minor, patch, prerelease, build })
            }
            _ => Err(Error::basic_parse("expected major.minor.patch for semver")),
        }
    }
}

impl SemanticVersion {
    /// Create a new `SemanticVersion` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::SemanticVersion> for SemanticVersion {
    fn from_protobuf(pb: services::SemanticVersion) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            major: pb.major as u32,
            minor: pb.minor as u32,
            patch: pb.patch as u32,
            prerelease: pb.pre,
            build: pb.build,
        })
    }
}

impl ToProtobuf for SemanticVersion {
    type Protobuf = services::SemanticVersion;
    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            major: self.major as i32,
            minor: self.minor as i32,
            patch: self.patch as i32,
            pre: self.prerelease.clone(),
            build: self.build.clone(),
        }
    }
}
