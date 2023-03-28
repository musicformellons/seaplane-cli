//! The `/restrict` endpoint APIs which allows working with [`Restriction`]s
pub mod models;

use std::str::FromStr;

use reqwest::{
    header::{self, CONTENT_TYPE},
    Url,
};

pub use self::models::*;
use crate::{
    api::{
        map_api_error,
        restrict::{error::RestrictError, RESTRICT_API_URL},
        shared::v1::RangeQueryContext,
        ApiRequest, RequestBuilder,
    },
    error::Result,
};

static RESTRICT_API_BASE_PATH: &str = "v1/restrict/";

/// A builder struct for creating a [`RestrictRequest`] which will then be used for making a
/// request against the `/restrict` APIs
#[derive(Debug)]
pub struct RestrictRequestBuilder {
    builder: RequestBuilder<RequestTarget>,
}

impl From<RequestBuilder<RequestTarget>> for RestrictRequestBuilder {
    fn from(builder: RequestBuilder<RequestTarget>) -> Self { Self { builder } }
}

impl Default for RestrictRequestBuilder {
    fn default() -> Self { Self::new() }
}

impl RestrictRequestBuilder {
    /// Create a new RestrictRequestBuilder
    pub fn new() -> Self { RequestBuilder::new(RESTRICT_API_URL, RESTRICT_API_BASE_PATH).into() }

    /// Build a RestrictRequest from the given parameters
    pub fn build(self) -> Result<RestrictRequest> { Ok(self.builder.build()?.into()) }

    /// Set the token used in Bearer Authorization
    ///
    /// **NOTE:** This is required for all endpoints
    #[must_use]
    pub fn token<U: Into<String>>(self, token: U) -> Self { self.builder.token(token).into() }

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

    // Used in testing and development to manually set the URL
    #[doc(hidden)]
    pub fn base_url<U: AsRef<str>>(self, url: U) -> Self { self.builder.base_url(url).into() }

    /// The restricted directory, encoded in url-safe base64.
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn single_restriction<S: Into<String>>(mut self, api: S, directory: S) -> Self {
        self.builder.target = Some(RequestTarget::Single {
            api: api.into(),
            directory: RestrictedDirectory::from_encoded(directory.into()),
        });
        self
    }

    /// The context with which to perform a range query within an API
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn api_range<S: Into<String>>(
        mut self,
        api: S,
        context: RangeQueryContext<RestrictedDirectory>,
    ) -> Self {
        self.builder.target = Some(RequestTarget::ApiRange { api: api.into(), context });
        self
    }

    /// The context with which to perform a range query across all APIs
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn all_range<S: Into<String>>(
        mut self,
        from_api: Option<S>,
        context: RangeQueryContext<RestrictedDirectory>,
    ) -> Self {
        self.builder.target =
            Some(RequestTarget::AllRange { from_api: from_api.map(|a| a.into()), context });
        self
    }
}

/// For making requests against the `/request` APIs.
#[derive(Debug)]
pub struct RestrictRequest {
    request: ApiRequest<RequestTarget>,
}

impl From<ApiRequest<RequestTarget>> for RestrictRequest {
    fn from(request: ApiRequest<RequestTarget>) -> Self { Self { request } }
}

impl RestrictRequest {
    /// Create a new request builder
    pub fn builder() -> RestrictRequestBuilder { RestrictRequestBuilder::new() }

    // Internal method creating the URL for single key endpoints
    fn single_url(&self) -> Result<Url> {
        match &self.request.target {
            Some(RequestTarget::Single { api, directory }) => Ok(self
                .request
                .endpoint_url
                .join(&format!("{}/base64:{}/", api, directory.encoded()))?),
            _ => Err(RestrictError::IncorrectRestrictRequestTarget)?,
        }
    }

    // Internal method creating the URL for all range endpoints
    fn range_url(&self) -> Result<Url> {
        match &self.request.target {
            Some(RequestTarget::AllRange { from_api, context }) => {
                let mut url = self.request.endpoint_url.clone();

                match (from_api, context.from()) {
                    (None, None) => Ok(url),
                    (Some(api), Some(from)) => {
                        url.set_query(Some(&format!(
                            "from_api={}&from=base64:{}",
                            api,
                            from.encoded()
                        )));
                        Ok(url)
                    }
                    (..) => Err(RestrictError::IncorrectRestrictRequestTarget)?,
                }
            }

            Some(RequestTarget::ApiRange { api, context }) => {
                let api = Api::from_str(api)
                    .map_err(|_| RestrictError::IncorrectRestrictRequestTarget)?;

                let mut url = self.request.endpoint_url.join(&format!("{api}/"))?;

                match context.from() {
                    None => Ok(url),
                    Some(from) => {
                        url.set_query(Some(&format!("from=base64:{}", from.encoded())));
                        Ok(url)
                    }
                }
            }
            _ => Err(RestrictError::IncorrectRestrictRequestTarget)?,
        }
    }

    /// Returns restriction details for an API-directory combination
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Single`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seaplane::api::restrict::v1::{RestrictRequest, RestrictRequestBuilder};
    ///
    /// let req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .single_restriction("config", "bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_restriction().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn get_restriction(&self) -> Result<Restriction> {
        let url = self.single_url()?;
        let resp = self
            .request
            .client
            .get(url)
            .bearer_auth(&self.request.token)
            .send()?;
        map_api_error(resp)?
            .json::<Restriction>()
            .map_err(Into::into)
    }

    /// Returns a single page of restrictions, starting from `from_api` and
    /// `from_key` combination.
    ///
    /// If more pages are desired, perform another range request using the
    /// `next_api` and `next_key` values from the first request as the
    /// `from_api` and `from_key` values of the following request, or use
    /// `get_all_pages`.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be an `ApiRange` or
    /// `AllRange`.
    ///
    /// # Examples
    ///
    /// ## Paging through single API restrictions
    ///
    /// ```no_run
    /// use seaplane::api::{
    ///     restrict::v1::{RestrictRequest, RestrictRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let context = RangeQueryContext::new();
    /// let req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .api_range("config", context)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_page().unwrap();
    /// dbg!(&resp);
    ///
    /// // To get next page:
    ///
    /// if let Some(next_key) = resp.next_key {
    ///     let mut context = RangeQueryContext::new();
    ///     context.set_from(next_key);
    ///
    ///     let req = RestrictRequestBuilder::new()
    ///         .token("abc123_token")
    ///         .api_range("config", context)
    ///         .build()
    ///         .unwrap();
    ///
    ///     let next_page_resp = req.get_page().unwrap();
    ///     dbg!(next_page_resp);
    /// }
    /// ```
    ///
    /// ## Paging through all restrictions
    ///
    /// ```no_run
    /// use seaplane::api::{
    ///     restrict::v1::{RestrictRequest, RestrictRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let context = RangeQueryContext::new();
    /// let req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .all_range::<String>(None, context)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_page().unwrap();
    /// dbg!(&resp);
    ///
    /// // To get next page:
    ///
    /// if let Some(next_key) = resp.next_key {
    ///     let api = resp.next_api.map(|a| a.to_string());
    ///     let mut context = RangeQueryContext::new();
    ///     context.set_from(next_key);
    ///
    ///     let req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .all_range::<String>(api, context)
    ///     .build()
    ///     .unwrap();
    ///
    ///     let next_page_resp = req.get_page().unwrap();
    ///     dbg!(next_page_resp);
    /// }

    /// ```
    pub fn get_page(&self) -> Result<RestrictionRange> {
        match &self.request.target {
            None | Some(RequestTarget::Single { .. }) => {
                Err(RestrictError::IncorrectRestrictRequestTarget)?
            }
            Some(RequestTarget::ApiRange { .. }) => {
                let url = self.range_url()?;

                let resp = self
                    .request
                    .client
                    .get(url)
                    .bearer_auth(&self.request.token)
                    .send()?;
                map_api_error(resp)?
                    .json::<RestrictionRange>()
                    .map_err(Into::into)
            }
            Some(RequestTarget::AllRange { .. }) => {
                let url = self.range_url()?;

                let resp = self
                    .request
                    .client
                    .get(url)
                    .bearer_auth(&self.request.token)
                    .send()?;
                map_api_error(resp)?
                    .json::<RestrictionRange>()
                    .map_err(Into::into)
            }
        }
    }

    /// Returns all restrictions within for a tenant or API.
    /// May perform multiple requests.
    ///
    /// If no directory is given, the root directory is used.
    /// If no `from` is given, the range begins from the start.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `ApiRange` or
    /// `AllRange`.
    ///
    /// # Examples
    ///
    /// ## Getting all restrictions for an API
    ///
    /// ```no_run
    /// use seaplane::api::{
    ///     restrict::v1::{RestrictRequest, RestrictRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let context = RangeQueryContext::new();
    /// let mut req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .api_range("config", context)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_all_pages().unwrap();
    /// dbg!(resp);
    /// ```
    ///
    /// ## Getting all restrictions across all APIs
    ///
    /// ```no_run
    /// use seaplane::api::{
    ///     restrict::v1::{RestrictRequest, RestrictRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let context = RangeQueryContext::new();
    /// let mut req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .all_range::<String>(None, context)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_all_pages().unwrap();
    /// dbg!(resp);
    /// ```

    // TODO: Replace this with a collect on a Pages/Entries iterator
    pub fn get_all_pages(&mut self) -> Result<Vec<Restriction>> {
        let mut pages = Vec::new();
        loop {
            let mut rr = self.get_page()?;
            pages.append(&mut rr.restrictions);
            if let Some(next_key) = rr.next_key {
                match &mut self.request.target {
                    None | Some(RequestTarget::Single { .. }) => {
                        Err(RestrictError::IncorrectRestrictRequestTarget)?
                    }
                    Some(RequestTarget::ApiRange { api: _, context }) => {
                        context.set_from(next_key);
                    }
                    Some(RequestTarget::AllRange { from_api: _, context }) => {
                        context.set_from(next_key);
                        self.request.target = Some(RequestTarget::AllRange {
                            from_api: rr.next_api.map(|a| a.to_string()),
                            context: context.to_owned(),
                        });
                    }
                }
            } else {
                break;
            }
        }
        Ok(pages)
    }

    /// Sets a restriction for an API-directory combination
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Single`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::str::FromStr;
    ///
    /// use seaplane::api::{
    ///     restrict::v1::{RestrictRequest, RestrictRequestBuilder, RestrictionDetails},
    ///     shared::v1::Region,
    /// };
    ///
    /// let req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .single_restriction("config", "bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let details = RestrictionDetails::builder()
    ///     .add_allowed_region(Region::from_str("xe").unwrap())
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.set_restriction(details).unwrap();
    /// dbg!(resp);
    /// ```
    pub fn set_restriction(&self, details: RestrictionDetails) -> Result<()> {
        let url = self.single_url()?;
        let resp = self
            .request
            .client
            .put(url)
            .bearer_auth(&self.request.token)
            .header(CONTENT_TYPE, header::HeaderValue::from_static("application/json"))
            .body(serde_json::to_string(&details)?)
            .send()?;
        map_api_error(resp)?
            .text()
            .map(|_| ()) // TODO: for now we drop the "success" message to control it ourselves
            .map_err(Into::into)
    }

    /// Removes a restriction for an API-directory combination
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Single`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seaplane::api::restrict::v1::{RestrictRequest, RestrictRequestBuilder};
    ///
    /// let req = RestrictRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .single_restriction("config", "bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.delete_restriction().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn delete_restriction(&self) -> Result<()> {
        let url = self.single_url()?;
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
}
