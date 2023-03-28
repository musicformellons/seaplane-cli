use thiserror::Error as ThisError;

use crate::rexports::seaplane_oid::error::Error as OidError;

#[derive(ThisError, Debug, Copy, Clone, PartialEq, Eq)]
pub enum FormationValidation {
    #[error("Formation name cannot be empty")]
    NameEmpty,
    #[error("Formation name too long, must be <= 63 in length")]
    NameLength,
    #[error("illegal character in Formation name; must only contain ASCII lowercase, digit, or hyphen ('-')")]
    NameInvalidChar,
    #[error("no more than three hyphens ('-') allowed in Formation name")]
    NameTooManyHyphens,
    #[error("consecutive hyphens ('--') not allowed in Formation name")]
    NameConsecutiveHyphens,
    #[error("Formation names may not start or end with a hyphen ('-')")]
    NameLeadingOrTrailingHyphen,
    #[error("request did not include the required formation name")]
    MissingFormationName,
    #[error("flights cannot be empty")]
    EmptyFlights,
    #[error("the given gateway flight either does not exist or is invalid")]
    InvalidGatewayFlight,
    #[error("missing required Flight name")]
    MissingFlightName,
    #[error("missing required Flight image reference")]
    MissingFlightImageReference,
    #[error("Request requires a valid Formation ID but none was provided")]
    MissingFormationId,
    #[error("a gateway flight was not specified or could not be implied because the number of flights is > 1")]
    NoGatewayFlight,
}

#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum ComputeError {
    #[error("{0}")]
    FormationValidation(#[from] FormationValidation),
    #[error("Object ID error: {0}")]
    Oid(#[from] OidError),
}
