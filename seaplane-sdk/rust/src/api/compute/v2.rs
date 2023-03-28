//! The `/formations` endpoint APIs which allows working with [`Formation`]s,
//! [`Flight`]s, and the underlying containers

mod models;
pub mod response;
mod validate;
pub use models::*;
pub use response::*;
pub use validate::*;

use crate::{
    api::{
        compute::{
            error::{ComputeError, FormationValidation},
            COMPUTE_API_URL,
        },
        error::map_api_error,
        ApiRequest, RequestBuilder,
    },
    error::Result,
};

const COMPUTE_API_ROUTE: &str = "v2beta/formations";

/// A builder struct for creating a [`FormationsRequest`] which will then be used for making a
/// request against the `/formations` APIs
#[derive(Debug)]
pub struct FormationsRequestBuilder {
    builder: RequestBuilder<FormationId>,
}

impl From<RequestBuilder<FormationId>> for FormationsRequestBuilder {
    fn from(builder: RequestBuilder<FormationId>) -> Self { Self { builder } }
}

impl Default for FormationsRequestBuilder {
    fn default() -> Self { Self::new() }
}

impl FormationsRequestBuilder {
    pub fn new() -> Self { RequestBuilder::new(COMPUTE_API_URL, COMPUTE_API_ROUTE).into() }

    /// Builds a FormationsRequest from the given parameters
    pub fn build(self) -> Result<FormationsRequest> { Ok(self.builder.build()?.into()) }

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

    /// The Object ID of the Formation to query as part of the request.
    ///
    /// **NOTE:** The Object ID is in the form of `frm-agc6amh7z527vijkv2cutplwaa` and is unique
    /// per Formation instance
    #[must_use]
    pub fn formation_id(self, oid: FormationId) -> Self { self.builder.target(oid).into() }
}

/// For making requests against the `/formations` APIs.
#[derive(Debug)]
pub struct FormationsRequest {
    request: ApiRequest<FormationId>,
}

impl From<ApiRequest<FormationId>> for FormationsRequest {
    fn from(request: ApiRequest<FormationId>) -> Self { Self { request } }
}

impl FormationsRequest {
    /// Create a new request builder
    pub fn builder() -> FormationsRequestBuilder { FormationsRequestBuilder::new() }

    /// Create a new Formation and returns the IDs of the created Formation.
    ///
    /// Uses `POST /formations`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use seaplane::api::compute::v2::{FormationsRequest, Formation, Flight};
    /// let req = FormationsRequest::builder()
    ///     .token("abc123")
    ///     .build()
    ///     .unwrap();
    ///
    /// let formation = Formation::builder()
    ///     .name("exmample-formation")
    ///     .add_flight(
    ///         Flight::builder()
    ///             .name("myflight")
    ///             .image("my/image:latest")
    ///             .build()
    ///             .unwrap(),
    ///     )
    ///     .build()
    ///     .unwrap();
    /// let resp = req.create(&formation).unwrap();
    /// dbg!(resp);
    /// ```
    pub fn create(&self, formation: &Formation) -> Result<CreateFormationResponse> {
        let req = self
            .request
            .client
            .post(self.request.endpoint_url.clone())
            .bearer_auth(&self.request.token)
            .json(formation);
        let resp = req.send()?;
        map_api_error(resp)?
            .json::<CreateFormationResponse>()
            .map_err(Into::into)
    }

    /// Deletes a formation
    ///
    /// Uses `DELETE /formations/ID`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use seaplane::api::compute::v2::{FormationsRequest};
    /// let req = FormationsRequest::builder()
    ///     .token("abc123")
    ///     .formation_id("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap())
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(req.delete().is_ok());
    /// ```
    pub fn delete(&self) -> Result<DeleteFormationResponse> {
        use FormationValidation::*;
        if self.request.target.is_none() {
            Err(ComputeError::FormationValidation(MissingFormationId))?;
        }
        let url = self
            .request
            .endpoint_url
            .join(&format!("formations/{}", self.oid()))?;
        let resp = self
            .request
            .client
            .delete(url)
            .bearer_auth(&self.request.token)
            .send()?;

        map_api_error(resp)?;
        Ok(())
    }

    // @TODO: a paging iterator may be more appropriate here in the future
    /// Returns a list of all the Formations you have access to
    ///
    /// Uses `GET /formations`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use seaplane::api::compute::v2::{FormationsRequest};
    /// let req = FormationsRequest::builder()
    ///     .token("abc123_token")
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get_all().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn get_all(&self) -> Result<GetFormationsResponse> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(self.request.endpoint_url.clone())
            .bearer_auth(&self.request.token)
            .send()?;

        map_api_error(resp)?
            .json::<GetFormationsResponse>()
            .map_err(Into::into)
    }

    /// Returns a single Formation's metadata
    ///
    /// Uses `GET /formations/ID`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use seaplane::api::compute::v2::{FormationsRequest};
    /// let req = FormationsRequest::builder()
    ///     .token("abc123_token")
    ///     .formation_id("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap())
    ///     .build()
    ///     .unwrap();
    ///
    /// let resp = req.get().unwrap();
    /// dbg!(resp);
    /// ```
    pub fn get(&self) -> Result<GetFormationResponse> {
        if self.request.target.is_none() {
            Err(ComputeError::FormationValidation(FormationValidation::MissingFormationId))?
        }
        let url = self
            .request
            .endpoint_url
            .join(&format!("formations/{}", self.oid()))?;
        let resp = self
            .request
            .client
            .get(url)
            .bearer_auth(&self.request.token)
            .send()?;

        map_api_error(resp)?
            .json::<GetFormationResponse>()
            .map_err(Into::into)
    }

    // Internal; gets the OID of the target formation
    //
    // # Panics
    //
    // If FormationId isn't `Some`
    #[inline]
    fn oid(&self) -> &FormationId { self.request.target.as_ref().unwrap() }
}
