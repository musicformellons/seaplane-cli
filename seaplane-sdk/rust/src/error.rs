//! Errors produced or propagated through the Seaplane SDK

use thiserror::Error;

#[cfg(all(feature = "compute_api_v2", feature = "unstable"))]
use crate::api::compute::v2::error::{ComputeRequest, FormationValidation};
use crate::api::ApiError;

pub type Result<T> = std::result::Result<T, SeaplaneError>;

#[derive(Error, Debug)]
pub enum SeaplaneError {
    #[error("http error: {0}")]
    UnknownHttp(reqwest::Error),
    #[error("{0}")]
    Decode(String),
    #[error("request did not include a required API key")]
    MissingRequestApiKey,
    #[error("request did not include a required authorization token")]
    MissingRequestAuthToken,
    #[error("request did not include the required formation name")]
    MissingFormationName,
    #[error("invalid URL")]
    UrlParse(#[from] url::ParseError),
    #[error("invalid json")]
    Json(#[from] serde_json::error::Error),
    #[error("request did not include any active configurations while force=false")]
    MissingActiveConfiguration,
    #[error("missing a required UUID")]
    MissingUuid,
    #[error("the given request conflict with one another")]
    ConflictingParams,
    #[error("flights cannot be empty")]
    EmptyFlights,
    #[error("a gateway flight was not specified or could not be implied because the number of flights is > 1")]
    NoGatewayFlight,
    #[error("the given gateway flight either does not exist or is invalid")]
    InvalidGatewayFlight,
    #[error("missing required Flight name")]
    MissingFlightName,
    #[error("missing required Flight image reference")]
    MissingFlightImageReference,
    #[error("the requirements specified in the builder are in conflict and invalid")]
    ConflictingRequirements,
    #[error("request did not include the required key")]
    MissingMetadataKey,
    #[error("request must target either key or range")]
    IncorrectMetadataRequestTarget,
    #[error("locks requests must target either a lock by name or a held lock")]
    IncorrectLocksRequestTarget,
    #[error("restrict requests must target all restrictions, an api, or an api and a key")]
    IncorrectRestrictRequestTarget,
    #[error("the API returned an error status")]
    ApiResponse(#[from] ApiError),
    #[cfg(all(feature = "compute_api_v2", feature = "unstable"))]
    #[error("{0}")]
    FormationValidation(#[from] FormationValidation),
    #[cfg(all(feature = "compute_api_v2", feature = "unstable"))]
    #[error("{0}")]
    ComputeRequest(#[from] ComputeRequest),
}

impl From<reqwest::Error> for SeaplaneError {
    fn from(re: reqwest::Error) -> Self {
        if re.is_decode() {
            SeaplaneError::Decode(re.to_string())
        } else {
            SeaplaneError::UnknownHttp(re)
        }
    }
}

impl PartialEq for SeaplaneError {
    fn eq(&self, rhs: &Self) -> bool {
        use SeaplaneError::*;

        match self {
            UnknownHttp(_) => matches!(rhs, UnknownHttp(_)),
            Decode(_) => matches!(rhs, Decode(_)),
            MissingRequestApiKey => matches!(rhs, MissingRequestApiKey),
            MissingRequestAuthToken => matches!(rhs, MissingRequestAuthToken),
            MissingFormationName => matches!(rhs, MissingFormationName),
            UrlParse(_) => matches!(rhs, UrlParse(_)),
            Json(_) => matches!(rhs, Json(_)),
            MissingActiveConfiguration => matches!(rhs, MissingActiveConfiguration),
            MissingUuid => matches!(rhs, MissingUuid),
            ConflictingParams => matches!(rhs, ConflictingParams),
            EmptyFlights => matches!(rhs, EmptyFlights),
            NoGatewayFlight => matches!(rhs, NoGatewayFlight),
            InvalidGatewayFlight => matches!(rhs, InvalidGatewayFlight),
            MissingFlightName => matches!(rhs, MissingFlightName),
            MissingFlightImageReference => matches!(rhs, MissingFlightImageReference),
            ConflictingRequirements => matches!(rhs, ConflictingRequirements),
            MissingMetadataKey => matches!(rhs, MissingMetadataKey),
            IncorrectMetadataRequestTarget => matches!(rhs, IncorrectMetadataRequestTarget),
            IncorrectLocksRequestTarget => matches!(rhs, IncorrectLocksRequestTarget),
            IncorrectRestrictRequestTarget => matches!(rhs, IncorrectRestrictRequestTarget),
            ApiResponse(ae) => match rhs {
                ApiResponse(oae) => ae == oae,
                _ => false,
            },
            #[cfg(all(feature = "compute_api_v2", feature = "unstable"))]
            FormationValidation(e) => {
                if let FormationValidation(re) = rhs {
                    e == re
                } else {
                    false
                }
            }
            #[cfg(all(feature = "compute_api_v2", feature = "unstable"))]
            ComputeRequest(e) => {
                if let ComputeRequest(re) = rhs {
                    e == re
                } else {
                    false
                }
            }
        }
    }
}
