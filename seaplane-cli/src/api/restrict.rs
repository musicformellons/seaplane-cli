use reqwest::Url;
use seaplane::{
    api::{
        identity::v1::AccessToken,
        restrict::v1::{
            RestrictRequest, RestrictedDirectory, Restriction, RestrictionDetails, RestrictionRange,
        },
        shared::v1::RangeQueryContext,
        ApiErrorKind,
    },
    error::SeaplaneError,
};

use crate::{
    api::request_token,
    context::Ctx,
    error::{CliError, Result},
};

/// Wraps an SDK `RestrictRequest` where we do additional things like re-use
/// request access tokens, and map errors appropriately.
#[derive(Debug)]
pub struct RestrictReq {
    api_key: String,
    api: Option<String>,
    directory: Option<String>,
    from_api: Option<String>,
    from_dir: Option<String>,
    token: Option<AccessToken>,
    inner: Option<RestrictRequest>,
    identity_url: Option<Url>,
    metadata_url: Option<Url>,
    insecure_urls: bool,
    invalid_certs: bool,
}

impl RestrictReq {
    pub fn new(ctx: &Ctx) -> Result<Self> {
        Ok(Self {
            api_key: ctx.args.api_key()?.into(),
            api: None,
            directory: None,
            from_api: None,
            from_dir: None,
            token: None,
            inner: None,
            identity_url: ctx.identity_url.clone(),
            metadata_url: ctx.metadata_url.clone(),
            #[cfg(feature = "allow_insecure_urls")]
            insecure_urls: ctx.insecure_urls,
            #[cfg(not(feature = "allow_insecure_urls"))]
            insecure_urls: false,
            #[cfg(feature = "allow_invalid_certs")]
            invalid_certs: ctx.invalid_certs,
            #[cfg(not(feature = "allow_invalid_certs"))]
            invalid_certs: false,
        })
    }

    pub fn set_api<S: Into<String>>(&mut self, api: S) -> Result<()> {
        self.api = Some(api.into());
        self.refresh_inner()
    }

    pub fn set_directory<S: Into<String>>(&mut self, dir: S) -> Result<()> {
        self.directory = Some(dir.into());
        self.refresh_inner()
    }

    pub fn set_from_api<S: Into<String>>(&mut self, api: S) -> Result<()> {
        self.from_api = Some(api.into());
        self.refresh_inner()
    }

    pub fn set_from_dir<S: Into<String>>(&mut self, dir: S) -> Result<()> {
        self.from_dir = Some(dir.into());
        self.refresh_inner()
    }

    /// Request a new Access Token
    pub fn refresh_token(&mut self) -> Result<()> {
        self.token = Some(request_token(
            &self.api_key,
            self.identity_url.as_ref(),
            self.insecure_urls,
            self.invalid_certs,
        )?);
        Ok(())
    }

    /// Re-build the inner `RestrictRequest`. This is mostly useful when one
    /// wants to point at different Restriction than the original request was
    /// pointed at. This method will also refresh the access token, only if
    /// required.
    fn refresh_inner(&mut self) -> Result<()> {
        let mut builder = RestrictRequest::builder().token(self.token_or_refresh()?);

        #[cfg(feature = "allow_insecure_urls")]
        {
            builder = builder.allow_http(self.insecure_urls);
        }
        #[cfg(feature = "allow_invalid_certs")]
        {
            builder = builder.allow_invalid_certs(self.invalid_certs);
        }

        if let Some(url) = &self.metadata_url {
            builder = builder.base_url(url);
        }

        match [&self.api, &self.directory] {
            [Some(api), Some(directory)] => builder = builder.single_restriction(api, directory),
            [Some(api), None] => {
                let mut context = RangeQueryContext::<RestrictedDirectory>::new();
                if let Some(from_dir) = &self.from_dir {
                    context.set_from(RestrictedDirectory::from_encoded(from_dir));
                };
                builder = builder.api_range(api, context)
            }
            [None, None] => {
                let mut context = RangeQueryContext::<RestrictedDirectory>::new();
                if let Some(from_dir) = &self.from_dir {
                    context.set_from(RestrictedDirectory::from_encoded(from_dir));
                };
                builder = builder.all_range(self.from_api.clone(), context)
            }
            [..] => {}
        };

        self.inner = Some(builder.build().map_err(CliError::from)?);
        Ok(())
    }

    /// Retrieves the JWT access token, requesting a new one if required.
    pub fn token_or_refresh(&mut self) -> Result<&str> {
        if self.token.is_none() {
            self.refresh_token()?;
        }
        Ok(&self.token.as_ref().unwrap().token)
    }
}

// Wrapped RestrictRequest methods to handle expired token retries
impl RestrictReq {
    pub fn get_restriction(&mut self) -> Result<Restriction> {
        maybe_retry!(self.get_restriction())
    }

    pub fn set_restriction(&mut self, details: RestrictionDetails) -> Result<()> {
        maybe_retry_cloned!(self.set_restriction(details))
    }
    pub fn delete_restriction(&mut self) -> Result<()> { maybe_retry!(self.delete_restriction()) }

    pub fn get_page(&mut self) -> Result<RestrictionRange> { maybe_retry!(self.get_page()) }
    pub fn get_all_pages(&mut self) -> Result<Vec<Restriction>> {
        maybe_retry!(self.get_all_pages())
    }
}
