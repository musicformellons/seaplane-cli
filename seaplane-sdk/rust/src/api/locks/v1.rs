//! The `/locks` endpoint APIs which allows working with [`HeldLock`]s
mod models;

use reqwest::Url;
use serde::Deserialize;

pub use self::models::*;
use crate::{
    api::{
        locks::{error::LocksError, LOCKS_API_URL},
        map_api_error,
        shared::v1::RangeQueryContext,
        ApiRequest, RequestBuilder,
    },
    base64::add_base64_path_segment,
    error::Result,
};

static LOCKS_API_BASE_PATH: &str = "v1/locks/";

/// A builder struct for creating a [`LocksRequest`] which will then be used for making a
/// request against the `/locks` APIs
#[derive(Debug)]
pub struct LocksRequestBuilder {
    builder: RequestBuilder<RequestTarget>,
}

/// For making requests against the `/locks` APIs.
#[derive(Debug)]
pub struct LocksRequest {
    request: ApiRequest<RequestTarget>,
}

impl From<RequestBuilder<RequestTarget>> for LocksRequestBuilder {
    fn from(builder: RequestBuilder<RequestTarget>) -> Self { Self { builder } }
}

impl Default for LocksRequestBuilder {
    fn default() -> Self { Self::new() }
}
impl LocksRequestBuilder {
    /// Create a new LocksRequestBuilder
    pub fn new() -> Self { RequestBuilder::new(LOCKS_API_URL, LOCKS_API_BASE_PATH).into() }

    /// Build a LocksRequest from the given parameters
    pub fn build(self) -> Result<LocksRequest> { Ok(self.builder.build()?.into()) }

    /// Allow non-HTTPS endpoints for this request (default: `false`)
    #[cfg(any(feature = "allow_insecure_urls", feature = "danger_zone"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "allow_insecure_urls", feature = "danger_zone"))))]
    pub fn allow_http(self, yes: bool) -> Self { self.builder.allow_http(yes).into() }

    /// Allow invalid TLS certificates (default: `false`)
    #[cfg(any(feature = "allow_invalid_certs", feature = "danger_zone"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "allow_invalid_certs", feature = "danger_zone"))))]
    pub fn allow_invalid_certs(self, yes: bool) -> Self {
        self.builder.allow_invalid_certs(yes).into()
    }

    /// Set the token used in Bearer Authorization
    ///
    /// **NOTE:** This is required for all endpoints
    #[must_use]
    pub fn token<U: Into<String>>(self, token: U) -> Self { self.builder.token(token).into() }

    // Used in testing and development to manually set the URL
    #[doc(hidden)]
    pub fn base_url<U: AsRef<str>>(self, url: U) -> Self { self.builder.base_url(url).into() }

    /// The lock name with which to perform operations where you may not be holding the lock.
    /// Encoded in url-safe base64
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn encoded_lock_name<S: Into<String>>(self, lock: S) -> Self {
        let name = LockName::from_encoded(lock.into());
        self.lock_name(name)
    }

    /// The lock name with which to perform operations where you may not be holding the lock.
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn lock_name(mut self, name: LockName) -> Self {
        self.builder.target = Some(RequestTarget::SingleLock(name));
        self
    }

    /// The held lock with which to perform held lock operations.
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn held_lock(mut self, lock: HeldLock) -> Self {
        self.builder.target = Some(RequestTarget::HeldLock(lock));
        self
    }

    /// The context with which to perform a range query
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn range(mut self, context: RangeQueryContext<LockName>) -> Self {
        self.builder.target = Some(RequestTarget::Range(context));
        self
    }
}

impl From<ApiRequest<RequestTarget>> for LocksRequest {
    fn from(request: ApiRequest<RequestTarget>) -> Self { Self { request } }
}

impl LocksRequest {
    /// Create a new request builder
    pub fn builder() -> LocksRequestBuilder { LocksRequestBuilder::new() }

    // Internal method creating the URL for all single lock endpoints
    fn single_lock_url(&self) -> Result<Url> {
        match &self.request.target {
            None | Some(RequestTarget::HeldLock(_) | RequestTarget::Range(_)) => {
                Err(LocksError::IncorrectLocksRequestTarget)?
            }
            Some(RequestTarget::SingleLock(l)) => {
                Ok(add_base64_path_segment(self.request.endpoint_url.clone(), l.encoded()))
            }
        }
    }

    // Internal method for creating the URL for held lock endpoints
    fn held_lock_url(&self) -> Result<Url> {
        match &self.request.target {
            None | Some(RequestTarget::SingleLock(_) | RequestTarget::Range(_)) => {
                Err(LocksError::IncorrectLocksRequestTarget)?
            }

            Some(RequestTarget::HeldLock(HeldLock { name, id, .. })) => {
                let mut url =
                    add_base64_path_segment(self.request.endpoint_url.clone(), name.encoded());
                url.set_query(Some(&format!("id={}", id.encoded())));
                Ok(url)
            }
        }
    }

    // Internal method for creating the URL for range endpoints
    fn range_url(&self) -> Result<Url> {
        match &self.request.target {
            None | Some(RequestTarget::SingleLock(_) | RequestTarget::HeldLock(_)) => {
                Err(LocksError::IncorrectLocksRequestTarget)?
            }
            Some(RequestTarget::Range(context)) => {
                let mut url = self.request.endpoint_url.clone();

                if let Some(encoded_dir) = context.directory() {
                    url = add_base64_path_segment(url, encoded_dir.encoded());
                    // A directory is distinguished from a key by the trailing slash
                    url.set_path(&format!("{}/", url.path()));
                }

                if let Some(from) = context.from() {
                    url.set_query(Some(&format!("from=base64:{}", from.encoded())));
                }

                Ok(url)
            }
        }
    }

    // Internal method for getting the lock name
    fn lock_name(&self) -> Result<LockName> {
        match &self.request.target {
            None | Some(RequestTarget::HeldLock(_) | RequestTarget::Range(_)) => {
                Err(LocksError::IncorrectLocksRequestTarget)?
            }
            Some(RequestTarget::SingleLock(l)) => Ok(l.clone()),
        }
    }

    /// Attempts to acquire the lock with the given lock name with the given TTL.
    /// Client-ID should identify the client making the request for debugging purposes.
    ///
    /// **NOTE:** This endpoints requires the `RequestTarget` be a `SingleLock`
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::locks::v1::{LocksRequest, LocksRequestBuilder};
    ///
    /// let req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_lock_name("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.acquire(15, "test-client").unwrap();
    /// dbg!(resp);
    /// ```
    pub fn acquire(&self, ttl: u32, client_id: &str) -> Result<HeldLock> {
        let mut url = self.single_lock_url()?;
        url.set_query(Some(&format!("ttl={ttl}&client-id={client_id}")));
        let resp = self
            .request
            .client
            .post(url)
            .bearer_auth(&self.request.token)
            .send()?;

        #[derive(Deserialize)]
        struct AcquireResponse {
            id: LockId,
            sequencer: u32,
        }

        let name = self.lock_name()?;
        map_api_error(resp)?
            .json::<AcquireResponse>()
            .map(|AcquireResponse { id, sequencer }| HeldLock { name, id, sequencer })
            .map_err(Into::into)
    }

    /// Attempts to release the given lock.
    ///
    /// **NOTE:** This endpoints requires the `RequestTarget` be a `HeldLock`
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::locks::v1::{LockId, LockName, LocksRequest, LocksRequestBuilder};
    /// // First we acquire the lock
    /// let req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_lock_name("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let acquired_lock = req.acquire(15, "test-client").unwrap();
    ///
    /// // Now it can be released
    /// let release_req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .held_lock(acquired_lock)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = release_req.release().unwrap();
    /// dbg!(resp)
    /// ```
    pub fn release(&self) -> Result<()> {
        let url = self.held_lock_url()?;

        let resp = self
            .request
            .client
            .delete(url)
            .bearer_auth(&self.request.token)
            .send()?;

        map_api_error(resp)?
            .text()
            .map(|_| ()) // TODO: for now we drop the "success" message to control it ourselves
            .map_err(Into::into)
    }

    /// Attempts to renew the given lock, setting the TTL to the given `ttl`
    ///
    /// **NOTE:** This endpoints requires the `RequestTarget` be a `HeldLock`
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::locks::v1::{LockId, LockName, LocksRequest, LocksRequestBuilder};
    /// // First we acquire the lock
    /// let req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_lock_name("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let acquired_lock = req.acquire(15, "test-client").unwrap();
    /// // Now it can be renewed with a new TTL of 20
    /// let renew_req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .held_lock(acquired_lock)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = renew_req.renew(20).unwrap();
    /// dbg!(resp)
    /// ```
    pub fn renew(&self, ttl: u32) -> Result<()> {
        let mut url = self.held_lock_url()?;

        url.query_pairs_mut().append_pair("ttl", &ttl.to_string());

        let resp = self
            .request
            .client
            .patch(url)
            .bearer_auth(&self.request.token)
            .send()?;

        map_api_error(resp)?
            .text()
            .map(|_| ()) // TODO: for now we drop the "success" message to control it ourselves
            .map_err(Into::into)
    }

    /// Gets information about a single lock.
    ///
    /// **NOTE:** This endpoints requires the `RequestTarget` be a `SingleLock`
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::locks::v1::{LocksRequest, LocksRequestBuilder};
    /// // First we acquire the lock
    /// let req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_lock_name("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_lock_info().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn get_lock_info(&self) -> Result<LockInfo> {
        let url = self.single_lock_url()?;

        let resp = self
            .request
            .client
            .get(url)
            .bearer_auth(&self.request.token)
            .send()?;

        map_api_error(resp)?.json::<LockInfo>().map_err(Into::into)
    }

    /// Returns a single page of lock information for the given directory, beginning with the `from`
    /// key.
    ///
    /// If no directory is given, the root directory is used.
    /// If no `from` is given, the range begins from the start.
    ///
    /// If more pages are desired, perform another range request using the `next` value from the
    /// first request as the `from` value of the following request, or use `get_all_pages`.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Range`.
    /// # Examples
    /// ```no_run
    /// use seaplane::api::{
    ///     locks::v1::{LockName, LocksRequest, LocksRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let root_dir_range: RangeQueryContext<LockName> = RangeQueryContext::new();
    ///
    /// let req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .range(root_dir_range)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_page().unwrap();
    ///
    /// if let Some(next_key) = resp.next {
    ///     let mut next_page_range = RangeQueryContext::new();
    ///     next_page_range.set_from(next_key);
    ///
    ///     let req = LocksRequestBuilder::new()
    ///         .token("abc123_token")
    ///         .range(next_page_range)
    ///         .build()
    ///         .unwrap();
    ///
    ///     let next_page_resp = req.get_page().unwrap();
    ///     dbg!(next_page_resp);
    /// }
    /// ```
    pub fn get_page(&self) -> Result<LockInfoRange> {
        match &self.request.target {
            None | Some(RequestTarget::SingleLock(_) | RequestTarget::HeldLock(_)) => {
                Err(LocksError::IncorrectLocksRequestTarget)?
            }
            Some(RequestTarget::Range(_)) => {
                let url = self.range_url()?;

                let resp = self
                    .request
                    .client
                    .get(url)
                    .bearer_auth(&self.request.token)
                    .send()?;
                map_api_error(resp)?
                    .json::<LockInfoRange>()
                    .map_err(Into::into)
            }
        }
    }

    /// Returns all held lock information for the given directory, from the `from` key onwards. May
    /// perform multiple requests.
    ///
    /// If no directory is given, the root directory is used.
    /// If no `from` is given, the range begins from the start.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Range`.
    /// # Examples
    /// ```no_run
    /// use seaplane::api::{
    ///     locks::v1::{LocksRequest, LocksRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let root_dir_range = RangeQueryContext::new();
    ///
    /// let mut req = LocksRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .range(root_dir_range)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_all_pages().unwrap();
    /// dbg!(resp);
    /// ```
    // TODO: Replace this with a collect on a Pages/Entries iterator
    pub fn get_all_pages(&mut self) -> Result<Vec<LockInfo>> {
        let mut pages = Vec::new();
        loop {
            let mut lir = self.get_page()?;
            pages.append(&mut lir.locks);
            if let Some(next_key) = lir.next {
                // TODO: Regrettable duplication here suggests that there should be a
                // ConfigKeyRequest and a ConfigRangeRequest
                if let Some(RequestTarget::Range(ref mut context)) = self.request.target {
                    context.set_from(next_key);
                } else {
                    Err(LocksError::IncorrectLocksRequestTarget)?
                }
            } else {
                break;
            }
        }
        Ok(pages)
    }
}
