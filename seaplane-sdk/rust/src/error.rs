//! Errors produced or propagated through the Seaplane SDK.
//!
//! The entrypoint is [SeaplaneError] which all API methods should use as their error type. There
//! are two categories of errors:
//!
//! * Errors encountered on the server side and returned (i.e. server-side)
//! * Errors encountered prior to sending the request to the server (i.e. client-side)
//!
//! All server-side errors should be represented as the [SeaplaneError::ApiResponse] variant which
//! wraps the [ApiError] type that expects server-side error responses to follow [RFC
//! 7807][rfc_7807]. There is a the minor exception of if the HTTP response could not be converted
//! into a [ApiError] due to an unknown or unimplemented status, in which case you should see
//! something like the [SeaplaneError::UnknownHttp] variant.
//!
//! Client-side errors are represented by the various enum variants. Errors specific to a
//! particular servicer are grouped by a dedicated variant wrapping that service's client-side
//! error type, such as [SeaplaneError::ComputeRequest] which wraps the client-side Compute service
//! error type ([ComputeError]) and contains errors specific to Compute requests.
//!
//! [rfc_7807]: https://www.rfc-editor.org/rfc/rfc7807

use thiserror::Error as ThisError;

use crate::api::ApiError;
#[cfg(feature = "compute_api_v2")]
use crate::api::{
    compute::error::ComputeError, locks::error::LocksError, metadata::error::MetadataError,
    restrict::error::RestrictError,
};

pub type Result<T> = std::result::Result<T, SeaplaneError>;

#[derive(ThisError, Debug)]
pub enum SeaplaneError {
    /// Server-side errors from all services
    #[error("{0}")]
    ApiResponse(#[from] ApiError),
    /// An unknown, or unimplemented HTTP response was found
    #[error("http error: {0}")]
    UnknownHttp(reqwest::Error),
    #[error("{0}")]
    UnknownRequest(reqwest::Error),
    #[error("request did not include a required API key")]
    MissingRequestApiKey,
    #[error("request did not include a required authorization token")]
    MissingRequestAuthToken,
    #[error("invalid URL")]
    UrlParse(#[from] url::ParseError),
    #[error("{0}")]
    Json(#[from] serde_json::error::Error),
    /// Client-side Compute Service Request Errors
    #[cfg(feature = "compute_api_v2")]
    #[error("{0}")]
    ComputeRequest(#[from] ComputeError),
    /// Client-side Locks Service Request Errors
    #[cfg(feature = "locks_api_v1")]
    #[error("{0}")]
    LocksRequest(#[from] LocksError),
    /// Client-side Metadata Service Request Errors
    #[cfg(feature = "metadata_api_v1")]
    #[error("{0}")]
    MetadataRequest(#[from] MetadataError),
    /// Client-side Restrict Service Request Errors
    #[cfg(feature = "restrict_api_v1")]
    #[error("{0}")]
    RestrictRequest(#[from] RestrictError),
}

impl From<reqwest::Error> for SeaplaneError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_builder() {
            SeaplaneError::UnknownRequest(e)
        } else {
            SeaplaneError::UnknownHttp(e)
        }
    }
}

impl PartialEq for SeaplaneError {
    fn eq(&self, rhs: &Self) -> bool {
        use SeaplaneError::*;

        match self {
            UnknownHttp(_) => matches!(rhs, UnknownHttp(_)),
            UnknownRequest(_) => matches!(rhs, UnknownRequest(_)),
            MissingRequestApiKey => matches!(rhs, MissingRequestApiKey),
            MissingRequestAuthToken => matches!(rhs, MissingRequestAuthToken),
            UrlParse(_) => matches!(rhs, UrlParse(_)),
            Json(_) => matches!(rhs, Json(_)),
            ApiResponse(ae) => match rhs {
                ApiResponse(oae) => ae == oae,
                _ => false,
            },
            #[cfg(feature = "compute_api_v2")]
            ComputeRequest(e) => {
                if let ComputeRequest(re) = rhs {
                    e == re
                } else {
                    false
                }
            }
            #[cfg(feature = "locks_api_v1")]
            LocksRequest(e) => {
                if let LocksRequest(re) = rhs {
                    e == re
                } else {
                    false
                }
            }
            #[cfg(feature = "metadata_api_v1")]
            MetadataRequest(e) => {
                if let MetadataRequest(re) = rhs {
                    e == re
                } else {
                    false
                }
            }
            #[cfg(feature = "restrict_api_v1")]
            RestrictRequest(e) => {
                if let RestrictRequest(re) = rhs {
                    e == re
                } else {
                    false
                }
            }
        }
    }
}
