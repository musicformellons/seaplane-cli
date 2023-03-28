use reqwest::Url;
use seaplane::{
    api::{
        compute::v2::{
            CreateFormationResponse, DeleteFormationResponse, Formation as FormationModel,
            FormationId, FormationsRequest, GetFormationResponse, GetFormationsResponse,
        },
        identity::v1::AccessToken,
    },
    error::SeaplaneError,
};

use crate::{
    api::request_token,
    context::Ctx,
    error::{CliError, Result},
};

/// Wraps an SDK `FormationsRequest` where we do additional things like re-use request access
/// tokens, allow changing the Formation this request is pointed to, and map errors appropriately.
#[derive(Debug)]
pub struct FormationsReq {
    api_key: String,
    oid: Option<FormationId>,
    token: Option<AccessToken>,
    inner: Option<FormationsRequest>,
    identity_url: Option<Url>,
    compute_url: Option<Url>,
    insecure_urls: bool,
    invalid_certs: bool,
}

impl FormationsReq {
    /// Builds a FormationsRequest and immediately requests an access token using the given API key.
    ///
    /// If the `oid` is `None` it should be noted that not all requests can be made without
    /// error. At present only is `FormationsRequest::create` and
    /// `FormationsRequest::get_all_formations`
    pub fn new<S: AsRef<str>>(ctx: &Ctx, oid: Option<S>) -> Result<Self> {
        let mut this = Self::new_delay_token(ctx)?;
        if let Some(oid_str) = oid {
            this.oid = Some(oid_str.as_ref().parse()?);
        }
        this.refresh_token()?;
        Ok(this)
    }

    /// Builds a FormationsRequest but *does not* request an access token using the given API key.
    ///
    /// You must call `refresh_token` to have the access token requested.
    pub fn new_delay_token(ctx: &Ctx) -> Result<Self> {
        Ok(Self {
            api_key: ctx.args.api_key()?.into(),
            oid: None,
            token: None,
            inner: None,
            identity_url: ctx.identity_url.clone(),
            compute_url: ctx.compute_url.clone(),
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

    /// Re-build the inner `FormationsRequest`. This is mostly useful when one wants to point at a
    /// different Formation than the original request was pointed at (i.e. via `set_name`). This
    /// method will also refresh the access token, only if required.
    fn refresh_inner(&mut self) -> Result<()> {
        let mut builder = FormationsRequest::builder().token(self.token_or_refresh()?);

        #[cfg(feature = "allow_insecure_urls")]
        {
            builder = builder.allow_http(self.insecure_urls);
        }
        #[cfg(feature = "allow_invalid_certs")]
        {
            builder = builder.allow_invalid_certs(self.invalid_certs);
        }

        if let Some(url) = &self.compute_url {
            builder = builder.base_url(url);
        }

        if let Some(oid) = &self.oid {
            builder = builder.formation_id(*oid);
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

    /// Sets the Formation ID and re-builds the inner FormationsRequest also requesting a new
    /// access token if required
    pub fn set_id_str<S: AsRef<str>>(&mut self, oid: S) -> Result<()> {
        self.oid = Some(oid.as_ref().parse()?);
        self.refresh_inner()
    }

    /// Sets the Formation ID and re-builds the inner FormationsRequest also requesting a new
    /// access token if required
    pub fn set_id(&mut self, oid: FormationId) -> Result<()> {
        self.oid = Some(oid);
        self.refresh_inner()
    }
}

// Wrapped FormationsRequest methods to handle expired token retries
impl FormationsReq {
    pub fn create(&mut self, formation: &FormationModel) -> Result<CreateFormationResponse> {
        maybe_retry!(self.create(formation))
    }
    pub fn delete(&mut self) -> Result<DeleteFormationResponse> { maybe_retry!(self.delete()) }
    pub fn get(&mut self) -> Result<GetFormationResponse> { maybe_retry!(self.get()) }
    pub fn get_all(&mut self) -> Result<GetFormationsResponse> { maybe_retry!(self.get_all()) }
}
