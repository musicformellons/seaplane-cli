//! An Object ID (OID) is a base32 (no padding) encoded UUID with a prefix
//! separated by a `-`.
//!
//! For example `tst-agc6amh7z527vijkv2cutplwaa`, by convention the prefix is three
//! ASCII lowercase characters, however that is a hard constraint of OIDs in
//! general. The current implementation limits prefixes to 3 characters, but prefix
//! limit could be exposed as a tunable if that need arises.
//!
//! ## The Pitch
//!
//! OIDs allow a "human readable subject line" in the form of the prefix, where
//! actual data is a UUID. This means while debugging or reviewing a system it's trivial to
//! determine if an incorrect OID was passed in a particular location by looking at the prefix.
//! This isn't easily achievable with bare UUIDs.
//!
//! Base32 encoding the UUID also allows compressing the data into a smaller and
//! more familiar format for humans, akin to a commit hash.
//!
//! ## The Anti-Pitch
//!
//! The downside to OIDs is a layer of indirection when handling IDs and values,
//! it's not immediately obvious that the OIDs are a prefixed UUID. Additionally,
//! the prefixes must be controlled in some manner including migrated on changes
//! which adds a layer of complexity at the application layer.
//!
//! There is also additional processing overhead compared to a bare UUID in order
//! to encode/decode as well as handling the appending and removing the prefixes.
//!
//! However, we believe the drawbacks to pale in comparison to the benefits derived
//! from the format.
//!
//! ## Example
//!
//! ```rust
//! use seaplane_oid::{error::*, Oid};
//! use uuid::Uuid;
//!
//! fn main() -> Result<()> {
//!     // OIDs can be created with a given prefix alone, which generates a new
//!     // UUID
//!     let oid = Oid::new("exm")?;
//!     println!("{oid}");
//!
//!     // OIDs can be parsed from strings, however the "value" must be a valid
//!     // base32 encoded UUID
//!     let oid: Oid = "tst-agc6amh7z527vijkv2cutplwaa".parse()?;
//!     println!("{oid}");
//!
//!     // OIDs can also be created from the raw parts
//!     let oid = Oid::with_uuid(
//!         "exm",
//!         "0185e030-ffcf-75fa-a12a-ae8549bd7600"
//!             .parse::<Uuid>()
//!             .unwrap(),
//!     )?;
//!
//!     // One can retrieve the various parts of the OID if needed
//!     println!("Prefix: {}", oid.prefix());
//!     println!("Value: {}", oid.value());
//!     println!("UUID: {}", oid.uuid());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## License
//!
//! Licensed under the Apache License, Version 2.0, Copyright 2023 Seaplane IO, Inc.
pub mod error;

use std::{any::type_name, fmt, marker::PhantomData, str::FromStr};

use uuid::Uuid;

use crate::error::{Error, Result};

macro_rules! base32_spec {
    () => {{
        static BASE32_SPEC: once_cell::sync::OnceCell<data_encoding::Encoding> =
            once_cell::sync::OnceCell::new();
        BASE32_SPEC.get_or_init(|| {
            let mut spec = data_encoding::Specification::new();
            spec.symbols.push_str("abcdefghijklmnopqrstuvwxyz234567");
            spec.encoding().unwrap()
        })
    }};
}

fn uuid_from_str(s: &str) -> Result<Uuid> {
    if s.is_empty() {
        return Err(Error::MissingValue);
    }
    Ok(Uuid::from_slice(&base32_spec!().decode(s.as_bytes())?)?)
}

/// An OID Prefix designed to be similar to a human readable "subject line" for the ID
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Prefix<const N: usize = 3> {
    bytes: [u8; N],
}

impl<const N: usize> fmt::Display for Prefix<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: self.bytes must not contain any invalid UTF-8. We don't expose the inner byte
        // array for manipulation, and the only way to construct self checks for a subset of ASCII
        // which itself is a subset of UTF-8
        unsafe { write!(f, "{}", std::str::from_utf8_unchecked(self.bytes.as_slice())) }
    }
}

impl<const N: usize> Prefix<N> {
    /// Create a Prefix from a slice of bytes. The bytes must be lowercase ASCII values of `0-9` or
    /// `a-z`, additionally the byte slice length must be equal to the prefix length.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        // Checking for ASCII 0-9,a-z
        if !slice
            .iter()
            .all(|&c| (c > b'/' && c < b':') || (c > b'`' && c < b'{'))
        {
            return Err(Error::InvalidPrefixChar);
        }
        if slice.len() != N {
            return Err(Error::PrefixByteLength);
        }
        let mut pfx = Prefix { bytes: [0_u8; N] };
        pfx.bytes.copy_from_slice(slice);
        Ok(pfx)
    }
}

impl<const N: usize> FromStr for Prefix<N> {
    type Err = Error;

    /// The string slice is converted to ASCII lowercase before creating the Prefix
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_slice(s.to_ascii_lowercase().as_bytes())
    }
}

impl<const N: usize> From<[u8; N]> for Prefix<N> {
    fn from(arr: [u8; N]) -> Self { Self { bytes: arr } }
}

impl<const N: usize> TryFrom<&[u8]> for Prefix<N> {
    type Error = Error;
    fn try_from(slice: &[u8]) -> std::result::Result<Self, Self::Error> { Self::from_slice(slice) }
}

impl<const N: usize> TryFrom<&str> for Prefix<N> {
    type Error = Error;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> { s.parse() }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    #[test]
    fn prefix_from_str() {
        let pfx = "frm".parse::<Prefix>();
        assert!(pfx.is_ok());
        assert_eq!(pfx.unwrap(), Prefix { bytes: [b'f', b'r', b'm'] });
    }

    #[test]
    fn prefix_from_str_err_len() {
        let pfx = "frmx".parse::<Prefix<3>>();
        assert!(pfx.is_err());
        assert_eq!(pfx.unwrap_err(), Error::PrefixByteLength);
    }

    #[test]
    fn prefix_from_str_err_char() {
        let pfx = "fr[".parse::<Prefix<3>>();
        assert!(pfx.is_err());
        assert_eq!(pfx.unwrap_err(), Error::InvalidPrefixChar);
    }

    #[test]
    fn prefix_from_str_uppercase_ok() {
        let pfx = "frM".parse::<Prefix>();
        assert!(pfx.is_ok());
        assert_eq!(pfx.unwrap(), Prefix { bytes: [b'f', b'r', b'm'] });
    }

    #[test]
    fn prefix_from_slice() {
        let arr: [u8; 3] = [b'f', b'r', b'm'];
        let pfx = Prefix::from_slice(arr.as_slice());
        assert!(pfx.is_ok());
        assert_eq!(pfx.unwrap(), Prefix { bytes: arr });
    }

    #[test]
    fn prefix_from_slice_err_len() {
        let arr: [u8; 4] = [b'f', b'r', b'm', b'x'];
        let pfx = Prefix::<3>::from_slice(arr.as_slice());
        assert!(pfx.is_err());
        assert_eq!(pfx.unwrap_err(), Error::PrefixByteLength);
    }

    #[test]
    fn prefix_from_slice_err_char() {
        let arr: [u8; 3] = [b'f', b'r', b']'];
        let pfx = Prefix::<3>::from_slice(arr.as_slice());
        assert!(pfx.is_err());
        assert_eq!(pfx.unwrap_err(), Error::InvalidPrefixChar);
    }

    #[test]
    fn prefix_from_slice_err_uppercase() {
        let arr: [u8; 3] = [b'f', b'r', b'M'];
        let pfx = Prefix::<3>::from_slice(arr.as_slice());
        assert!(pfx.is_err());
        assert_eq!(pfx.unwrap_err(), Error::InvalidPrefixChar);
    }

    #[test]
    fn prefix_to_string() {
        let pfx: Prefix = "frM".parse().unwrap();
        assert_eq!("frm".to_string(), pfx.to_string());
    }
}

/// An Object ID
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Oid {
    prefix: Prefix<3>,
    uuid: Uuid,
}

impl Oid {
    /// Create a new OID with a given [`Prefix`] and generating a new UUID
    ///
    /// **NOTE:** The Prefix must be 3 ASCII characters (this restriction is arbitrary and could be
    /// lifted in the future by exposing an API to tune the [`Prefix`] length)
    pub fn new<P>(prefix: P) -> Result<Self>
    where
        P: TryInto<Prefix, Error = Error>,
    {
        Self::with_uuid(prefix, Uuid::new_v4())
    }

    /// Create a new OID with a given [`Prefix`] and a given UUID. If the UUID is not a version 7
    /// an error isr returned.
    ///
    /// **NOTE:** The Prefix must be 3 ASCII characters (this restriction is arbitrary and could be
    /// lifted in the future by exposing an API to tune the [`Prefix`] length)
    pub fn with_uuid<P>(prefix: P, uuid: Uuid) -> Result<Self>
    where
        P: TryInto<Prefix, Error = Error>,
    {
        Ok(Self { prefix: prefix.try_into()?, uuid })
    }

    /// Get the [`Prefix`] of the OID
    pub fn prefix(&self) -> Prefix { self.prefix }

    /// Get the value portion of the  of the OID, which is the base32 encoded string following the
    /// `-` separator
    pub fn value(&self) -> String { base32_spec!().encode(self.uuid.as_bytes()) }

    /// Get the UUID of the OID
    pub fn uuid(&self) -> &Uuid { &self.uuid }
}

impl FromStr for Oid {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((pfx, val)) = s.split_once('-') {
            if pfx.is_empty() {
                return Err(Error::MissingPrefix);
            }

            return Ok(Self { prefix: pfx.parse()?, uuid: uuid_from_str(val)? });
        }

        Err(Error::MissingSeparator)
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.prefix, self.value())
    }
}

#[cfg(feature = "serde")]
impl ::serde::Serialize for Oid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::ser::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> ::serde::Deserialize<'de> for Oid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::de::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(::serde::de::Error::custom)
    }
}

#[cfg(test)]
mod oid_tests {
    use wildmatch::WildMatch;

    use super::*;

    #[test]
    fn oid_to_str() -> Result<()> {
        let oid = Oid::new("tst")?;
        assert!(WildMatch::new("tst-??????????????????????????").matches(&oid.to_string()));
        Ok(())
    }

    #[test]
    fn str_to_oid() {
        let res = "tst-agc6amh7z527vijkv2cutplwaa".parse::<Oid>();
        assert_eq!(
            res.unwrap(),
            Oid {
                prefix: "tst".parse().unwrap(),
                uuid: "0185e030-ffcf-75fa-a12a-ae8549bd7600".parse().unwrap(),
            }
        );
    }

    #[test]
    fn str_to_oid_err_prefix() {
        let res = "-agc6amh7z527vijkv2cutplwaa".parse::<Oid>();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::MissingPrefix);
    }

    #[test]
    fn str_to_oid_err_value() {
        let res = "tst-".parse::<Oid>();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::MissingValue);
    }

    #[test]
    fn str_to_oid_err_decode() {
        let res = "tst-&gc6amh7z527vijkv2ctplwaa".parse::<Oid>();
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::Base32Decode(_)));
    }

    #[test]
    fn str_to_oid_err_no_sep() {
        let res = "agc6amh7z527vijkv2cutplwaa".parse::<Oid>();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::MissingSeparator);
    }

    #[test]
    fn str_to_oid_err_two_sep() {
        let res = "tst-agc6amh7z-527vijkv2ctplwaa".parse::<Oid>();
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::Base32Decode(_)));
    }

    #[test]
    fn oid_to_uuid() {
        let oid: Oid = "tst-agc6amh7z527vijkv2cutplwaa".parse().unwrap();
        assert_eq!(
            oid.uuid(),
            &"0185e030-ffcf-75fa-a12a-ae8549bd7600"
                .parse::<Uuid>()
                .unwrap()
        );
    }
}

pub trait OidPrefix {
    fn string_prefix() -> String {
        type_name::<Self>()
            .split(':')
            .last()
            .map(|s| s.to_ascii_lowercase())
            .unwrap()
    }
}

/// A Typed Object ID where the Prefix is part of the type
///
/// # Examples
///
/// A nice property of this two different prefix are two different types, and thus the following
/// fails to compile:
///
/// ```compile_fail
/// struct A;
/// impl OidPrefix for A {}
///
/// struct B;
/// impl OidPrefix for B {}
///
/// // The same UUID for both
/// let uuid = Uuid::new_v4();
/// let oid_a: TypedOid<A> = TypedOid::with_uuid(uuid.clone());
/// let oid_b: TypedOid<B> = TypedOid::with_uuid(uuid);
///
/// // This fails to compile because `TypedOid<A>` is a different type than `TypedOid<B>` and no
/// // PartialEq or Eq is implemented between these two types. The same would hold as function
/// // parameters, etc.
/// oid_a == oid_b
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypedOid<P: OidPrefix> {
    uuid: Uuid,
    _prefix: PhantomData<P>,
}

impl<P: OidPrefix> TypedOid<P> {
    /// Create a new TypedOid with a random UUID
    pub fn new() -> Self { Self::with_uuid(Uuid::new_v4()) }

    /// Create a new TypedOid with a given UUID
    pub fn with_uuid(uuid: Uuid) -> Self { Self { uuid, _prefix: PhantomData } }

    /// Get the [`Prefix`] of the OID
    ///
    /// # Panics
    ///
    /// If the Type `P` translates to an invalid prefix
    pub fn prefix(&self) -> Prefix {
        Prefix::from_str(&P::string_prefix()).expect("Invalid Prefix")
    }

    /// Get the value portion of the  of the OID, which is the base32 encoded string following the
    /// `-` separator
    pub fn value(&self) -> String { base32_spec!().encode(self.uuid.as_bytes()) }

    /// Get the UUID of the OID
    pub fn uuid(&self) -> &Uuid { &self.uuid }
}

impl<P: OidPrefix> Default for TypedOid<P> {
    fn default() -> Self { Self::new() }
}

impl<P: OidPrefix> fmt::Display for TypedOid<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", P::string_prefix(), self.value())
    }
}

impl<P: OidPrefix> FromStr for TypedOid<P> {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((pfx, val)) = s.split_once('-') {
            if pfx.is_empty() {
                return Err(Error::MissingPrefix);
            }

            if pfx != P::string_prefix() {
                return Err(Error::InvalidPrefixChar);
            }

            return Ok(Self { uuid: uuid_from_str(val)?, _prefix: PhantomData });
        }

        Err(Error::MissingSeparator)
    }
}

#[cfg(test)]
mod typed_oid_tests {
    use wildmatch::WildMatch;

    use super::*;

    #[test]
    fn typed_oid() {
        #[derive(Debug)]
        struct Tst;
        impl OidPrefix for Tst {}

        let oid: TypedOid<Tst> = TypedOid::new();
        assert!(
            WildMatch::new("tst-??????????????????????????").matches(&oid.to_string()),
            "{oid}"
        );

        let res = "tst-5wacbutjwbdexonddvdb2lnyxu".parse::<TypedOid<Tst>>();
        assert!(res.is_ok());
        let oid: TypedOid<Tst> = res.unwrap();
        assert_eq!(
            oid.uuid(),
            &"ed8020d2-69b0-464b-b9a3-1d461d2db8bd"
                .parse::<Uuid>()
                .unwrap()
        );

        let res = "frm-5wacbutjwbdexonddvdb2lnyxu".parse::<TypedOid<Tst>>();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::InvalidPrefixChar);
    }
}
