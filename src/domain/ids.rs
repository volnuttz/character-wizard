//! Closed SRD identifiers used by the canonical character record.

use std::{fmt, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

macro_rules! srd_id {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
        pub enum $name {
            $(#[serde(rename = $value)] $variant),+
        }

        impl $name {
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }

        impl Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target { self.as_str() }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool { self.as_str() == *other }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $($value => Ok(Self::$variant),)+ _ => Err(format!("unknown SRD {}: {value}", stringify!($name))) }
            }
        }
    };
}

/// Built-in SRD class names and stable external pack class IDs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ClassId(String);

impl ClassId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ClassId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for ClassId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialEq<&str> for ClassId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl FromStr for ClassId {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim().is_empty() {
            Err("class identifier must not be empty".to_owned())
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

/// Built-in SRD background names and stable external pack background IDs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct BackgroundId(String);

impl BackgroundId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for BackgroundId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for BackgroundId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialEq<&str> for BackgroundId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl FromStr for BackgroundId {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim().is_empty() {
            Err("background identifier must not be empty".to_owned())
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

/// Built-in SRD species names and stable external pack species IDs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct SpeciesId(String);

impl SpeciesId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for SpeciesId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for SpeciesId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialEq<&str> for SpeciesId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl FromStr for SpeciesId {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim().is_empty() {
            Err("species identifier must not be empty".to_owned())
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

srd_id!(Size {
    Small => "Small", Medium => "Medium",
});
