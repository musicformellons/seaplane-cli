use std::collections::HashSet;

use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, Engine, GeneralPurpose},
};
use seaplane::api::{
    restrict::v1::{RestrictedDirectory, RestrictionDetails},
    shared::v1::{Provider as ProviderModel, Region as RegionModel},
};

use crate::{
    cli::cmds::restrict::{
        common::{Provider, Region},
        SeaplaneRestrictCommonArgMatches, SeaplaneRestrictListArgMatches,
        SeaplaneRestrictSetArgMatches,
    },
    error::Result,
};

/// Represents the "Source of Truth" i.e. it combines all the CLI options, ENV vars, and config
/// values into a single structure that can be used later to build models for the API or local
/// structs for serializing
// TODO: we may not want to derive this we implement circular references
#[derive(Debug, Default, Clone)]
pub struct RestrictCtx {
    pub api: Option<String>,
    /// A base64 encoded directory
    pub directory: Option<RestrictedDirectory>,
    pub from_api: Option<String>,
    /// A base64 encoded key
    pub from_dir: Option<String>,
    /// Use actual API model since that is ultimately what we want
    pub providers_allowed: HashSet<ProviderModel>,
    /// Use actual API model since that is ultimately what we want
    pub providers_denied: HashSet<ProviderModel>,
    /// Use actual API model since that is ultimately what we want
    pub regions_allowed: HashSet<RegionModel>,
    /// Use actual API model since that is ultimately what we want
    pub regions_denied: HashSet<RegionModel>,
    /// Is the directory already URL safe base64 encoded
    pub base64: bool,
    /// Print with decoding
    pub decode: bool,
    /// Skip the headers in --format=table
    pub no_header: bool,
}

impl RestrictCtx {
    /// Builds a RestictCtx from ArgMatches
    pub fn from_restrict_common(matches: &SeaplaneRestrictCommonArgMatches) -> Result<RestrictCtx> {
        let matches = matches.0;
        let base64 = matches.get_flag("base64");
        let api = matches.get_one::<String>("api").unwrap();
        let dir = matches.get_one::<String>("directory").unwrap();

        Ok(RestrictCtx {
            api: Some(api.into()),
            directory: if base64 {
                // Check that what the user passed really is valid base64
                let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
                let _ = engine.decode(dir)?;
                Some(RestrictedDirectory::from_encoded(dir))
            } else {
                Some(RestrictedDirectory::from_unencoded(dir))
            },
            base64: true, // At this point all keys and values should be encoded as base64
            ..RestrictCtx::default()
        })
    }

    /// Builds a RestictCtx from ArgMatches
    pub fn from_restrict_list(matches: &SeaplaneRestrictListArgMatches) -> Result<RestrictCtx> {
        let api = matches.0.get_one::<String>("api").map(|a| a.to_owned());

        Ok(RestrictCtx { api, ..RestrictCtx::default() })
    }

    /// Builds a RestictCtx from ArgMatches
    pub fn from_restrict_set(matches: &SeaplaneRestrictSetArgMatches) -> Result<RestrictCtx> {
        let matches = matches.0;
        let base64 = matches.get_flag("base64");
        let raw_api = matches.get_one::<String>("api").unwrap();
        let raw_dir = matches.get_one::<String>("directory").unwrap();

        let providers_allowed: HashSet<ProviderModel> = matches
            .get_many::<Provider>("provider")
            .unwrap_or_default()
            .filter_map(Provider::into_model)
            .collect();
        let providers_denied: HashSet<ProviderModel> = matches
            .get_many::<Provider>("exclude-provider")
            .unwrap_or_default()
            .filter_map(Provider::into_model)
            .collect();
        let regions_allowed: HashSet<RegionModel> = matches
            .get_many::<Region>("region")
            .unwrap_or_default()
            .filter_map(Region::into_model)
            .collect();
        let regions_denied: HashSet<RegionModel> = matches
            .get_many::<Region>("exclude-region")
            .unwrap_or_default()
            .filter_map(Region::into_model)
            .collect();
        Ok(RestrictCtx {
            api: Some(raw_api.into()),
            directory: if base64 {
                // Check that what the user passed really is valid base64
                let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
                let _unused = engine.decode(raw_dir)?;
                Some(RestrictedDirectory::from_encoded(raw_dir))
            } else {
                Some(RestrictedDirectory::from_unencoded(raw_dir))
            },
            base64: true, // At this point all keys and values should be encoded as base64
            providers_allowed,
            providers_denied,
            regions_allowed,
            regions_denied,
            ..RestrictCtx::default()
        })
    }

    pub fn restriction_details(&self) -> Result<RestrictionDetails> {
        let mut builder = RestrictionDetails::builder();

        for item in &self.providers_allowed {
            builder = builder.add_allowed_provider(*item);
        }
        for item in &self.providers_denied {
            builder = builder.add_denied_provider(*item);
        }
        for item in &self.regions_allowed {
            builder = builder.add_allowed_region(*item);
        }
        for item in &self.regions_denied {
            builder = builder.add_denied_region(*item);
        }

        Ok(builder.build()?)
    }
}
