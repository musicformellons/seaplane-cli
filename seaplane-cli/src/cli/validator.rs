//! Utility functions used to validate various argument types on the CLI. The arguments and return
//! type are what the `clap::Arg::validator` expects.

use std::{path::Path, result::Result as StdResult};

use crate::{
    context::{FlightCtx, DEFAULT_IMAGE_REGISTRY_URL},
    error::CliErrorKind,
    ops::validator::validate_name,
};

/// The arg can be any of:
///
/// - name
/// - SPEC for Flights
/// - '-' (means STDIN)
/// - path
pub fn validate_path_inline(s: &str) -> StdResult<String, String> {
    validate_name(s)
        .or_else(|_| validate_path(s))
        .or_else(|_| validate_stdin(s))
        .or_else(|_| validate_inline_flight_spec(s))
        .map_err(|s| {
            format!("the value must be a SPEC|PATH|- where PATH is a valid path or - opens STDIN\n\ncaused by: {s}")
        })
}

/// The value may be `path` where `path` is some path that exists
pub fn validate_path(path: &str) -> StdResult<String, String> {
    if !Path::exists(path.as_ref()) {
        #[cfg(not(any(feature = "semantic_ui_tests", feature = "ui_tests")))]
        return Err(format!("path '{path}' does not exist"));
    }

    Ok(path.into())
}

/// The value may be `-`
pub fn validate_stdin(s: &str) -> StdResult<String, &'static str> {
    if s != "-" {
        return Err("the value '-' was not provided");
    }

    Ok(s.into())
}

/// The value may be:
///
/// name=NAME,image=IMG,maximum=NUM,minimum=NUM,api-permission[=true|false],architecture=ARCH
///
/// where only image is required, and architecture can be passed multiple times.
pub fn validate_inline_flight_spec(s: &str) -> StdResult<String, String> {
    // We use the default image registry URL regardless of what the user has set because we're only
    // checking validity, not actually using this data
    if let Err(e) = FlightCtx::from_inline_flight(s, DEFAULT_IMAGE_REGISTRY_URL) {
        return Err(format!("invalid Fligth SPEC: {}", {
            match e.kind() {
                CliErrorKind::InlineFlightUnknownItem(s) => format!("unknown item {s}"),
                CliErrorKind::InlineFlightMissingValue(s) => {
                    format!("key '{s}' is missing the value")
                }
                CliErrorKind::InlineFlightHasSpace => {
                    String::from("inline flight contains a space (' ')")
                }
                CliErrorKind::InlineFlightMissingImage => {
                    String::from("Missing required image=IMAGE-SPEC")
                }
                CliErrorKind::InlineFlightInvalidName(s) => {
                    format!("Flight name '{s}' isn't valid")
                }
                CliErrorKind::ImageReference(e) => format!("invalid IMAGE-SPEC: {e}"),
                _ => unreachable!(),
            }
        }));
    }
    Ok(s.into())
}
