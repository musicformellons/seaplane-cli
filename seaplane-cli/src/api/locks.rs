use reqwest::Url;
use seaplane::{
    api::{
        identity::v0::AccessToken,
        locks::v1::{
            HeldLock as HeldLockModel, LockId, LockInfo as LockInfoModel, LockInfoRange, LockName,
            LocksRequest, LocksRequestBuilder,
        },
        shared::v1::{Directory, RangeQueryContext},
        ApiErrorKind,
    },
    error::SeaplaneError,
};

use crate::{
    api::request_token,
    context::Ctx,
    error::{CliError, Result},
};

/// Wraps an SDK `LocksRequest` where we do additional things like re-use request access
/// tokens, allow changing the Locks this request is pointed to, and map errors appropriately.
#[derive(Debug)]
pub struct LocksReq {
    api_key: String,
    lock_id: Option<String>,
    name: Option<LockName>,
    token: Option<AccessToken>,
    inner: Option<LocksRequest>,
    identity_url: Option<Url>,
    locks_url: Option<Url>,
    insecure_urls: bool,
    invalid_certs: bool,
}

impl LocksReq {
    pub fn new(ctx: &Ctx) -> Result<Self> {
        Ok(Self {
            api_key: ctx.args.api_key()?.into(),
            lock_id: None,
            name: None,
            token: None,
            inner: None,
            identity_url: ctx.identity_url.clone(),
            locks_url: ctx.locks_url.clone(),
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

    pub fn set_identifiers<S: Into<String>>(
        &mut self,
        name: Option<LockName>,
        lock_id: Option<S>,
    ) -> Result<()> {
        self.name = name;
        self.lock_id = lock_id.map(|s| s.into());
        self.refresh_inner()
    }

    pub fn set_name(&mut self, name: LockName) -> Result<()> {
        self.name = Some(name);
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

    /// Re-build the inner `LocksRequest`. This is mostly useful when one wants to point at a
    /// different Lock than the original request was pointed at (i.e. via `set_name`). This
    /// method will also refresh the access token, only if required.
    fn refresh_inner(&mut self) -> Result<()> {
        let mut builder = LocksRequest::builder().token(self.token_or_refresh()?);

        #[cfg(feature = "allow_insecure_urls")]
        {
            builder = builder.allow_http(self.insecure_urls);
        }
        #[cfg(feature = "allow_invalid_certs")]
        {
            builder = builder.allow_invalid_certs(self.invalid_certs);
        }
        if self.name.is_none() {
            panic!("all LocksRequests must have a name")
        }

        match &self.lock_id {
            Some(lock_id) => {
                let default_sequencer_value = 0u32;
                builder = builder.held_lock(HeldLockModel::new(
                    self.name.clone().unwrap(),
                    LockId::from_encoded(lock_id),
                    default_sequencer_value,
                ));
            }
            None => builder = builder.lock_name(self.name.clone().unwrap()),
        }

        if let Some(url) = &self.locks_url {
            builder = builder.base_url(url);
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

    /// Gets a page of held locks from `dir` if present (or the root) if not, optionally starting
    /// from `next_key`
    pub fn get_page(
        &mut self,
        next_key: Option<LockName>,
        dir: Option<LockName>,
    ) -> Result<LockInfoRange> {
        // get_page doesn't use `inner` here, since it doesn't refer to any lock name
        // (Specifically, get_page() doesn't refer to any individual lock)
        let mut range = RangeQueryContext::new();
        if let Some(k) = next_key {
            range.set_from(k);
        }

        if let Some(d) = dir {
            range.set_directory(Directory::from_encoded(d.encoded()));
        }

        let mut builder = LocksRequestBuilder::new()
            .token(self.token_or_refresh()?)
            .range(range.clone());
        #[cfg(feature = "allow_insecure_urls")]
        {
            builder = builder.allow_http(self.insecure_urls);
        }
        #[cfg(feature = "allow_invalid_certs")]
        {
            builder = builder.allow_invalid_certs(self.invalid_certs);
        }

        if let Some(url) = &self.locks_url {
            builder = builder.base_url(url);
        }

        let req = builder.build().unwrap();

        match req.get_page() {
            Err(SeaplaneError::ApiResponse(ae)) if ae.kind == ApiErrorKind::Unauthorized => {
                self.token = Some(request_token(
                    &self.api_key,
                    self.identity_url.as_ref(),
                    self.insecure_urls,
                    self.invalid_certs,
                )?);
                let next_req = LocksRequestBuilder::new()
                    .token(self.token_or_refresh()?)
                    .range(range)
                    .build()
                    .unwrap();

                Ok(next_req.get_page()?)
            }
            result => result.map_err(CliError::from),
        }
    }
}

// Wrapped LocksRequest methods to handle expired token retries
//
impl LocksReq {
    pub fn acquire(&mut self, ttl: u32, client_id: &str) -> Result<HeldLockModel> {
        maybe_retry!(self.acquire(ttl, client_id))
    }
    pub fn release(&mut self) -> Result<()> { maybe_retry!(self.release()) }
    pub fn renew(&mut self, ttl: u32) -> Result<()> { maybe_retry!(self.renew(ttl)) }
    pub fn get_lock_info(&mut self) -> Result<LockInfoModel> { maybe_retry!(self.get_lock_info()) }
}
