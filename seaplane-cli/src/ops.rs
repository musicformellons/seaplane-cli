//! This module provides types that wrap the API endpoint models and add additional fields/context
//! that is only relevant for the CLI or purposes of consuming the API.

pub mod db;
pub mod encoded_string;
pub mod flight;
pub mod formation;
pub mod locks;
pub mod metadata;
pub mod restrict;
pub mod state_version;
pub mod validator;

use std::{fmt, str::FromStr};

pub use crate::ops::encoded_string::EncodedString;
use crate::ops::validator::validate_name;

// An enum representing either a valid Formation Name or a valid OID
#[derive(Debug, Clone)]
pub enum NameId<T> {
    Name(String),
    Oid(T),
}

impl<T> NameId<T> {
    pub fn is_name(&self) -> bool { matches!(self, NameId::Name(_)) }

    pub fn is_oid(&self) -> bool { matches!(self, NameId::Oid(_)) }

    pub fn name(&self) -> Option<&str> {
        if let NameId::Name(s) = self {
            return Some(s);
        }
        None
    }

    pub fn oid(&self) -> Option<&T> {
        if let NameId::Oid(t) = self {
            return Some(t);
        }
        None
    }
}

impl<T> FromStr for NameId<T>
where
    T: FromStr,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<T>() {
            Ok(val) => Ok(NameId::Oid(val)),
            Err(_) => Ok(NameId::Name(validate_name(s)?)),
        }
    }
}

impl<T> fmt::Display for NameId<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameId::Name(name) => write!(f, "{name}"),
            NameId::Oid(oid) => write!(f, "{oid}"),
        }
    }
}

pub fn generate_name() -> String {
    // TODO: Maybe set an upper bound on the number of iterations and don't expect
    names::Generator::default()
        .find(|name| validate_name(name).is_ok())
        .expect("Failed to generate a random name")
}
