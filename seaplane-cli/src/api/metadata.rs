use reqwest::Url;
use seaplane::{
    api::{
        identity::v1::AccessToken,
        metadata::v1::{
            Key, KeyValue as KeyValueModel, KeyValueRange as KeyValueRangeModel, MetadataRequest,
            Value as ValueModel,
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

/// Wraps an SDK `MetadataRequest` where we do additional things like re-use request access
/// tokens, allow changing the Formation this request is pointed to, and map errors appropriately.
#[derive(Debug)]
pub struct MetadataReq {
    api_key: String,
    key: Option<String>,
    range: Option<RangeQueryContext<Key>>,
    token: Option<AccessToken>,
    inner: Option<MetadataRequest>,
    identity_url: Option<Url>,
    metadata_url: Option<Url>,
    insecure_urls: bool,
    invalid_certs: bool,
}

impl MetadataReq {
    pub fn new(ctx: &Ctx) -> Result<Self> {
        Ok(Self {
            api_key: ctx.args.api_key()?.into(),
            key: None,
            range: None,
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

    pub fn set_key<S: Into<String>>(&mut self, key: S) -> Result<()> {
        self.key = Some(key.into());
        self.range = None;
        self.refresh_inner()
    }

    pub fn set_dir(&mut self, dir: RangeQueryContext<Key>) -> Result<()> {
        self.range = Some(dir);
        self.key = None;
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

    /// Re-build the inner `MetadataRequest`. This is mostly useful when one wants to point at
    /// different Metadata than the original request was pointed at. This method will also refresh
    /// the access token, only if required.
    fn refresh_inner(&mut self) -> Result<()> {
        let mut builder = MetadataRequest::builder().token(self.token_or_refresh()?);

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

        if let Some(key) = &self.key {
            builder = builder.encoded_key(key);
        }

        if let Some(range) = &self.range {
            builder = builder.range(range.clone());
        }

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

// Wrapped MetadataRequest methods to handle expired token retries
impl MetadataReq {
    pub fn get_value(&mut self) -> Result<ValueModel> { maybe_retry!(self.get_value()) }
    pub fn put_value_unencoded<S: AsRef<[u8]>>(&mut self, value: S) -> Result<()> {
        maybe_retry!(self.put_value_unencoded(value.as_ref()))
    }
    pub fn put_value(&mut self, value: ValueModel) -> Result<()> {
        maybe_retry_cloned!(self.put_value(value))
    }
    pub fn delete_value(&mut self) -> Result<()> { maybe_retry!(self.delete_value()) }
    pub fn get_page(&mut self) -> Result<KeyValueRangeModel> { maybe_retry!(self.get_page()) }
    pub fn get_all_pages(&mut self) -> Result<Vec<KeyValueModel>> {
        maybe_retry_cloned!(self.get_all_pages())
    }
}
