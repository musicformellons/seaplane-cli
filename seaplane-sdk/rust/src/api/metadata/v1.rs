//! The `/config` endpoint APIs which allows working with [`KeyValue`]s
mod models;

use reqwest::{
    header::{self, CONTENT_TYPE},
    Url,
};

pub use self::models::*;
use crate::{
    api::{
        map_api_error,
        metadata::{error::MetadataError, METADATA_API_URL},
        shared::v1::RangeQueryContext,
        ApiRequest, RequestBuilder,
    },
    base64::add_base64_path_segment,
    error::Result,
};

const METADATA_API_BASE_PATH: &str = "v1/config/";

/// A builder struct for creating a [`MetadataRequest`] which will then be used for making a
/// request against the `/config` APIs
#[derive(Debug)]
pub struct MetadataRequestBuilder {
    builder: RequestBuilder<RequestTarget>,
}

impl From<RequestBuilder<RequestTarget>> for MetadataRequestBuilder {
    fn from(builder: RequestBuilder<RequestTarget>) -> Self { Self { builder } }
}

impl Default for MetadataRequestBuilder {
    fn default() -> Self { Self::new() }
}

impl MetadataRequestBuilder {
    /// Create a new MetadataRequestBuilder
    pub fn new() -> Self { RequestBuilder::new(METADATA_API_URL, METADATA_API_BASE_PATH).into() }

    /// Build an MetadataRequest from the given parameters
    pub fn build(self) -> Result<MetadataRequest> { Ok(self.builder.build()?.into()) }

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

    /// The key with which to query the store, encoded in url-safe base64.
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn encoded_key<S: Into<String>>(mut self, key: S) -> Self {
        self.builder.target = Some(RequestTarget::Key(Key::from_encoded(key.into())));
        self
    }

    /// The context with which to perform a range query
    ///
    /// **NOTE:** This is not required for all endpoints
    #[must_use]
    pub fn range(mut self, context: RangeQueryContext<Key>) -> Self {
        self.builder.target = Some(RequestTarget::Range(context));
        self
    }
}

/// For making requests against the `/config` APIs.
#[derive(Debug)]
pub struct MetadataRequest {
    request: ApiRequest<RequestTarget>,
}

impl From<ApiRequest<RequestTarget>> for MetadataRequest {
    fn from(request: ApiRequest<RequestTarget>) -> Self { Self { request } }
}

impl MetadataRequest {
    /// Create a new request builder
    pub fn builder() -> MetadataRequestBuilder { MetadataRequestBuilder::new() }

    // Internal method creating the URL for all single key endpoints
    fn single_key_url(&self) -> Result<Url> {
        match &self.request.target {
            None | Some(RequestTarget::Range(_)) => {
                Err(MetadataError::IncorrectMetadataRequestTarget)?
            }
            Some(RequestTarget::Key(k)) => {
                Ok(add_base64_path_segment(self.request.endpoint_url.clone(), k.encoded()))
            }
        }
    }

    // Internal method creating the URL for range endpoints
    fn range_url(&self) -> Result<Url> {
        match &self.request.target {
            None | Some(RequestTarget::Key(_)) => {
                Err(MetadataError::IncorrectMetadataRequestTarget)?
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

    /// Returns the key value pair associated with the set key.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Key`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seaplane::api::metadata::v1::{MetadataRequest, MetadataRequestBuilder};
    ///
    /// let req = MetadataRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_key("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_value().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn get_value(&self) -> Result<Value> {
        let url = self.single_key_url()?;
        let resp = self
            .request
            .client
            .get(url)
            .bearer_auth(&self.request.token)
            .send()?;
        map_api_error(resp)?
            .json::<KeyValue>()
            .map(|kv| kv.value)
            .map_err(Into::into)
    }

    /// Adds an unencoded value to the store at the given key performing the encoding before
    /// sending the request.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Key`.
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::metadata::v1::{MetadataRequest, MetadataRequestBuilder, Value};
    ///
    /// let req = MetadataRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_key("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.put_value_unencoded("I'll be encoded!").unwrap();
    /// dbg!(resp);
    /// ```
    pub fn put_value_unencoded<S: AsRef<[u8]>>(&self, value: S) -> Result<()> {
        self.put_value(Value::from_unencoded(value))
    }

    /// Adds a base64 encoded value to the store at the given key.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Key`.
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::metadata::v1::{MetadataRequest, MetadataRequestBuilder, Value};
    ///
    /// let req = MetadataRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_key("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.put_value(Value::from_encoded("YWhhYgo")).unwrap();
    /// dbg!(resp);
    /// ```
    pub fn put_value(&self, value: Value) -> Result<()> {
        let url = self.single_key_url()?;
        let resp = self
            .request
            .client
            .put(url)
            .bearer_auth(&self.request.token)
            .header(CONTENT_TYPE, header::HeaderValue::from_static("application/octet-stream"))
            .body(value.to_string())
            .send()?;
        map_api_error(resp)?
            .text()
            .map(|_| ()) // TODO: for now we drop the "success" message to control it ourselves
            .map_err(Into::into)
    }

    /// Deletes the key value pair at from a given base64 encoded key.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Key`.
    ///
    /// # Examples
    /// ```no_run
    /// use seaplane::api::metadata::v1::{MetadataRequest, MetadataRequestBuilder};
    ///
    /// let req = MetadataRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .encoded_key("bW9ieQo")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.delete_value().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn delete_value(&self) -> Result<()> {
        let url = self.single_key_url()?;
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

    /// Returns a single page of key value pairs for the given directory, beginning with the `from`
    /// key.
    ///
    /// If no directory is given, the root directory is used.
    /// If no `from` is given, the range begins from the start.
    ///
    /// If more pages are desired, perform another range request using the `next_key` value from the
    /// first request as the `from` value of the following request, or use `get_all_pages`.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Range`.
    /// # Examples
    /// ```no_run
    /// use seaplane::api::{
    ///     metadata::v1::{MetadataRequest, MetadataRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let root_dir_range = RangeQueryContext::new();
    ///
    /// let req = MetadataRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .range(root_dir_range)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_page().unwrap();
    ///
    /// if let Some(next_key) = resp.next_key {
    ///     let mut next_page_range = RangeQueryContext::new();
    ///     next_page_range.set_from(next_key);
    ///
    ///     let req = MetadataRequestBuilder::new()
    ///         .token("abc123_token")
    ///         .range(next_page_range)
    ///         .build()
    ///         .unwrap();
    ///
    ///     let next_page_resp = req.get_page().unwrap();
    ///     dbg!(next_page_resp);
    /// }
    /// ```
    pub fn get_page(&self) -> Result<KeyValueRange> {
        match &self.request.target {
            None | Some(RequestTarget::Key(_)) => {
                Err(MetadataError::IncorrectMetadataRequestTarget)?
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
                    .json::<KeyValueRange>()
                    .map_err(Into::into)
            }
        }
    }

    /// Returns all key-value pairs for the given directory, from the `from` key onwards. May
    /// perform multiple requests.
    ///
    /// If no directory is given, the root directory is used.
    /// If no `from` is given, the range begins from the start.
    ///
    /// **NOTE:** This endpoint requires the `RequestTarget` be a `Range`.
    /// # Examples
    /// ```no_run
    /// use seaplane::api::{
    ///     metadata::v1::{MetadataRequest, MetadataRequestBuilder},
    ///     shared::v1::RangeQueryContext,
    /// };
    ///
    /// let root_dir_range = RangeQueryContext::new();
    ///
    /// let mut req = MetadataRequestBuilder::new()
    ///     .token("abc123_token")
    ///     .range(root_dir_range)
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_all_pages().unwrap();
    /// dbg!(resp);
    /// ```
    // TODO: Replace this with a collect on a Pages/Entries iterator
    pub fn get_all_pages(&mut self) -> Result<Vec<KeyValue>> {
        let mut pages = Vec::new();
        loop {
            let mut kvr = self.get_page()?;
            pages.append(&mut kvr.kvs);
            if let Some(next_key) = kvr.next_key {
                // TODO: Regrettable duplication here suggests that there should
                // be a MetadataKeyRequest and a MetadataRangeRequest
                if let Some(RequestTarget::Range(ref mut context)) = self.request.target {
                    context.set_from(next_key);
                } else {
                    Err(MetadataError::IncorrectMetadataRequestTarget)?
                }
            } else {
                break;
            }
        }
        Ok(pages)
    }
}
