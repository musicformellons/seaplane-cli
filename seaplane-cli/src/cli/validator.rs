//! Utility functions used to validate various argument types on the CLI. The arguments and return
//! type are what the `clap::Arg::validator` expects.

use std::{path::Path, result::Result as StdResult};

use crate::ops::formation::Endpoint;

/// Ensures a valid Endpoint
pub fn validate_endpoint(s: &str) -> StdResult<(), String> {
    if s.parse::<Endpoint>().is_err() {
        return Err("invalid endpoint".to_string());
    }
    Ok(())
}

/// The arg can be any of:
///
/// - name
/// - Local ID (32byte hex encoded string)
/// - @- (means STDIN)
/// - @path
pub fn validate_name_id_path<F>(name_validator: F, s: &str) -> StdResult<(), String>
where
    F: Fn(&str) -> StdResult<(), &'static str>,
{
    name_validator(s)
        .or_else(|_| validate_id(s))
        .or_else(|_| validate_at_path(s))
        .or_else(|_| validate_at_stdin(s))
        .map_err(|_| {
            "the value must be a NAME|ID|@PATH|@- where @PATH is a valid path or @- opens STDIN"
                .to_owned()
        })
}

/// The arg can be any of:
///
/// - name
/// - Local ID (32byte hex encoded string)
pub fn validate_name_id<F>(name_validator: F, s: &str) -> StdResult<(), &'static str>
where
    F: Fn(&str) -> StdResult<(), &'static str>,
{
    name_validator(s).or_else(|_| validate_id(s))
}

/// Rules:
///
/// - 1-63 alphanumeric characters (only ASCII lowercase) and hyphens ( 0-9, a-z, A-Z, and '-' )
/// (aka, one DNS segment)
pub fn validate_flight_name(name: &str) -> StdResult<(), &'static str> {
    if name.is_empty() {
        return Err("Flight name cannot be empty");
    }
    if name.len() > 63 {
        return Err("Flight name too long, must be <= 63 in length");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(
            "illegal character in Flight name; must only contain ASCII lowercase, digit, or hyphen ('-')",
        );
    }
    if name.contains("--") {
        return Err("repeated hyphens ('--') not allowed in Flight name");
    }

    Ok(())
}

/// Current Rules:
///
///  - 1-30 alphanumeric (only ASCII lowercase) characters or hyphen (0-9, a-z, A-Z, and '-' )
///  - hyphens ('-') may not be repeated (i.e. '--')
///  - no more than three (3) total hyphens
///  - no consecutive hyphens
///  - no trailing hyphen
pub fn validate_formation_name(name: &str) -> StdResult<(), &'static str> {
    if name.is_empty() {
        return Err("Formation name cannot be empty");
    }
    if name.len() > 30 {
        return Err("Formation name too long, must be <= 30 in length");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(
            "illegal character in Formation name; must only contain ASCII lowercase, digit, or hyphen ('-')",
        );
    }
    if name.chars().filter(|c| *c == '-').count() > 3 {
        return Err("no more than three hyphens ('-') allowed in Formation name");
    }
    if name.contains("--") {
        return Err("repeated hyphens ('--') not allowed in Formation name");
    }
    if name.chars().last() == Some('-') {
        return Err("Formation names may not end with a hyphen ('-')");
    }

    Ok(())
}

/// The value may be `@path` where `path` is some path that exists
pub fn validate_at_path(s: &str) -> StdResult<(), String> {
    if let Some(path) = s.strip_prefix('@') {
        if !Path::exists(path.as_ref()) {
            return Err(format!("path '{path}' does not exist"));
        }
    } else {
        return Err("the '@<path>'  was not provided".to_owned());
    }

    Ok(())
}

/// The value may be `@-`
pub fn validate_at_stdin(s: &str) -> StdResult<(), &'static str> {
    if s != "@-" {
        return Err("the value '@-' was not provided");
    }

    Ok(())
}

/// The value may be a *up to* a 32byte hex encoded string
pub fn validate_id(s: &str) -> StdResult<(), &'static str> {
    if s.is_empty() {
        return Err("ID cannot be empty");
    }
    if !s.chars().all(is_hex_char) {
        return Err("found non-hex character");
    }
    if s.chars().count() > 64 {
        return Err("ID provided is too long");
    }

    Ok(())
}

fn is_hex_char(c: char) -> bool {
    matches!(c, 'a'..='f' | 'A'..='F' | '0'..='9')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_hex_char_valid() {
        for c in ('a'..'g').chain(('A'..'G').chain('0'..='9')) {
            assert!(is_hex_char(c));
        }
    }

    #[test]
    fn is_hex_char_invalid() {
        for c in ('g'..='z').chain('G'..'Z') {
            assert!(!is_hex_char(c));
        }
    }

    #[test]
    fn invalid_flight_names() {
        assert!(validate_flight_name("").is_err());
        assert!(validate_flight_name("no-special-chars!").is_err());
        assert!(validate_flight_name("imwaaaaaytoolongforanythingthatshouldbeanameimwaaaaaytoolongforanythingthatshouldbeaname").is_err());
        assert!(validate_flight_name("noUperCase").is_err());
    }

    #[test]
    fn invalid_formation_names() {
        assert!(validate_formation_name("").is_err());
        assert!(validate_formation_name("no-special-chars!").is_err());
        assert!(validate_formation_name("imwaaaaaytoolongforanythingthatshouldbeaname").is_err());
        assert!(validate_formation_name("too-many-hyphens-in-here").is_err());
        assert!(validate_formation_name("no-ending-hyphen-").is_err());
        assert!(validate_formation_name("noUperCase").is_err());
    }

    #[test]
    fn invalid_id() {
        assert!(validate_id("").is_err());
        assert!(validate_id("imnotahexvalue").is_err());
        // Too long
        assert!(validate_id("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").is_err());
    }

    #[test]
    fn invalid_at_path() {
        assert!(validate_id("").is_err());
        assert!(validate_id("@").is_err());
        assert!(validate_id("@-").is_err());
        assert!(validate_id("@foo").is_err());
        assert!(validate_id("foo").is_err());
    }

    #[test]
    fn invalid_at_stdin() {
        assert!(validate_id("").is_err());
        assert!(validate_id("@").is_err());
        assert!(validate_id("@foo").is_err());
        assert!(validate_id("foo").is_err());
    }
}
