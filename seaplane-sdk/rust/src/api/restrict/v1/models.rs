use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString, EnumVariantNames};

use crate::{
    api::{
        restrict::error::RestrictError,
        shared::v1::{Provider, RangeQueryContext, Region},
    },
    base64::Base64Encoded,
    error::SeaplaneError,
    impl_base64,
};

/// The target of a request. It can be either a single restriction or a list of
/// restrictions
#[non_exhaustive]
#[derive(Debug)]
pub(crate) enum RequestTarget {
    Single { api: String, directory: RestrictedDirectory },
    ApiRange { api: String, context: RangeQueryContext<RestrictedDirectory> },
    AllRange { from_api: Option<String>, context: RangeQueryContext<RestrictedDirectory> },
}

/// The response given from a range query
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestrictionRange {
    /// A lower bound of the next page of results
    pub next_api: Option<Api>,
    pub next_key: Option<RestrictedDirectory>,
    /// The range of Restrictions
    pub restrictions: Vec<Restriction>,
}

/// Contains information about the restricted API, directory, restriction
/// details and state
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Restriction {
    pub api: Api,
    pub directory: RestrictedDirectory,
    pub details: RestrictionDetails,
    pub state: RestrictionState,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    AsRefStr,
    strum::Display,
    EnumString,
    EnumVariantNames,
)]
#[strum(ascii_case_insensitive)]
#[non_exhaustive]
pub enum Api {
    Config,
    Locks,
}

impl_deser_from_str!(Api);

#[cfg(test)]
mod test_api {
    use super::*;

    #[test]
    fn api_case_insensitive() {
        let api: Api = serde_json::from_str("\"config\"").unwrap();
        assert_eq!(api, Api::Config);
        let api: Api = serde_json::from_str("\"Config\"").unwrap();
        assert_eq!(api, Api::Config);
        let api: Api = serde_json::from_str("\"CONFIG\"").unwrap();
        assert_eq!(api, Api::Config);
    }
}
/// A key pointing to a directory, encoded in url-safe base64.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct RestrictedDirectory {
    inner: Base64Encoded,
}
impl_base64!(RestrictedDirectory);

impl AsRef<str> for RestrictedDirectory {
    fn as_ref(&self) -> &str { self.inner.as_ref() }
}

impl fmt::Display for RestrictedDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.inner) }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, strum::Display, EnumString, EnumVariantNames,
)]
#[strum(ascii_case_insensitive)]
pub enum RestrictionState {
    Pending,
    Enforced,
}

impl_deser_from_str!(RestrictionState);

/// A builder for creating a [`RestrictionDetails`] struct
#[derive(Debug, Default)]
pub struct RestrictionDetailsBuilder {
    providers_allowed: BTreeSet<Provider>,
    providers_denied: BTreeSet<Provider>,
    regions_allowed: BTreeSet<Region>,
    regions_denied: BTreeSet<Region>,
}

impl RestrictionDetailsBuilder {
    /// Add a [`Provider`] which is allowed for use when placing data.
    ///
    /// If this conflicts with `providers_denied`
    /// ([`RestrictionDetailsBuilder::add_denied_provider`]) (e.g. [`Provider::GCP`] is both
    /// allowed and denied) the configuration is invalid and will be rejected.
    ///
    /// **NOTE:** This method can be called multiple times. All values will be utilized.
    #[must_use]
    pub fn add_allowed_provider<P: Into<Provider>>(mut self, provider: P) -> Self {
        self.providers_allowed.insert(provider.into());
        self
    }

    /// The inverse of [`RestrictionDetailsBuilder::add_allowed_provider`] which specifies
    /// [`Provider`] which is not allowed for use when placing data.
    ///
    /// By default no [`Provider`]s are denied.
    ///
    /// If this conflicts with `providers_allowed`
    /// ([`RestrictionDetailsBuilder::add_allowed_provider`]) (e.g. [`Provider::GCP`] is both
    /// allowed and denied) the configuration is invalid and will be rejected.
    ///
    /// **NOTE:** This method can be called multiple times. All values will be utilized.
    #[must_use]
    pub fn add_denied_provider<P: Into<Provider>>(mut self, provider: P) -> Self {
        self.providers_denied.insert(provider.into());
        self
    }

    /// Add a [`Region`] which is allowed for use when placing data.
    ///
    /// If this conflicts with `regions_denied`
    /// ([`RestrictionDetailsBuilder::add_denied_region`]) (e.g. [`Region::XN`] is both
    /// allowed and denied) the configuration is invalid and will be rejected.
    ///
    /// **NOTE:** This method can be called multiple times. All values will be utilized.
    #[must_use]
    pub fn add_allowed_region<R: Into<Region>>(mut self, region: R) -> Self {
        self.regions_allowed.insert(region.into());
        self
    }

    /// The inverse of [`RestrictionDetailsBuilder::add_allowed_region`] which specifies
    /// [`Region`] which is not allowed for use when placing data.
    ///
    /// By default no [`Region`]s are denied.
    ///
    /// If this conflicts with `regions_allowed`
    /// ([`RestrictionDetailsBuilder::add_allowed_region`]) (e.g. [`Region::XN`] is both
    /// allowed and denied) the configuration is invalid and will be rejected.
    ///
    /// **NOTE:** This method can be called multiple times. All values will be utilized.
    #[must_use]
    pub fn add_denied_region<R: Into<Region>>(mut self, region: R) -> Self {
        self.regions_denied.insert(region.into());
        self
    }

    /// Performs validation checks, and builds the instance of [`RestrictionDetails`]
    pub fn build(self) -> Result<RestrictionDetails, SeaplaneError> {
        if (self
            .providers_allowed
            .intersection(&self.providers_denied)
            .count()
            + self
                .regions_allowed
                .intersection(&self.regions_denied)
                .count())
            > 0
        {
            Err(RestrictError::ConflictingRequirements)?
        }

        Ok(RestrictionDetails {
            providers_allowed: self.providers_allowed,
            providers_denied: self.providers_denied,
            regions_allowed: self.regions_allowed,
            regions_denied: self.regions_denied,
        })
    }
}
/// Defines limits on where data can be stored.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RestrictionDetails {
    #[serde(default)]
    pub regions_allowed: BTreeSet<Region>,
    #[serde(default)]
    pub regions_denied: BTreeSet<Region>,
    #[serde(default)]
    pub providers_allowed: BTreeSet<Provider>,
    #[serde(default)]
    pub providers_denied: BTreeSet<Provider>,
}

impl RestrictionDetails {
    /// Create a [`RestrictionDetailsBuilder`] to build a new `RestrcitionDetails`
    pub fn builder() -> RestrictionDetailsBuilder { RestrictionDetailsBuilder::default() }
}

/// The response given from a range query
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestrictionsRange {
    /// A lower bound of the next page of results
    pub next_key: Option<RestrictedDirectory>,
    /// The range of key value pairs returned
    pub restrictions: Vec<Restriction>,
}
