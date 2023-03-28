//! Errors that come from the API endpoints

use std::{error::Error, fmt, result::Result as StdResult};

use reqwest::blocking::Response;
use serde::{de, Deserialize};
use serde_json::{Map, Value};

use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiResponse {
    pub kind: Option<String>,
    pub title: String,
    pub detail: Option<String>,
    pub status: Option<usize>,
    // the catch all for additional values
    pub meta: Map<String, Value>,
}

impl<'de> serde::Deserialize<'de> for ApiResponse {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let mut meta = Map::deserialize(deserializer)?;
        let kind = meta
            .remove("type")
            .map_or(Ok(None), Deserialize::deserialize)
            .map_err(de::Error::custom)?;
        let title = meta
            .remove("title")
            .ok_or_else(|| de::Error::missing_field("title"))
            .map(Deserialize::deserialize)?
            .map_err(de::Error::custom)?;
        let detail = meta
            .remove("detail")
            .map_or(Ok(None), Deserialize::deserialize)
            .map_err(de::Error::custom)?;
        let status = meta
            .remove("status")
            .map_or(Ok(None), Deserialize::deserialize)
            .map_err(de::Error::custom)?;
        Ok(ApiResponse { kind, title, detail, status, meta })
    }
}

impl fmt::Display for ApiResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.title)?;
        if let Some(msg) = &self.detail {
            writeln!(f, "{msg}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod api_response_error_tests {
    use super::*;

    #[test]
    fn deserialize() {
        let j = r#" {"title":"something"} "#;
        let _: ApiResponse = serde_json::from_str(j).unwrap();

        let j = r#" {"title":"something","type":"/error","detail":"more something","status":400} "#;
        let _: ApiResponse = serde_json::from_str(j).unwrap();

        let j = r#" {"title":"something","type":"/error","detail":"more something","status":400,"unknown":{"other":"field"}} "#;
        let _: ApiResponse = serde_json::from_str(j).unwrap();
    }
}

/// Maps a response error for all of the coordination services that use a JSON response type
pub fn map_api_error(resp: Response) -> Result<Response> {
    if let Err(source) = resp.error_for_status_ref() {
        return Err(ApiError { response: resp.json::<ApiResponse>()?, source }.into());
    }
    Ok(resp)
}

/// A generic Server Side error response derived from errors following [RFC 7807][rfc_7807]
///
/// [rfc_7807]: https://www.rfc-editor.org/rfc/rfc7807
#[derive(Debug)]
#[non_exhaustive]
pub struct ApiError {
    pub response: ApiResponse,
    pub source: reqwest::Error,
}

impl ApiError {
    /// A convenience method for seeing if the error came from an HTTP 404 NOT FOUND
    pub fn is_http_not_found(&self) -> bool {
        self.source.status() == Some(reqwest::StatusCode::NOT_FOUND)
    }

    /// A convenience method for seeing if the error came from an HTTP 401 UNAUTHORIZED
    pub fn is_http_unauthorized(&self) -> bool {
        self.source.status() == Some(reqwest::StatusCode::UNAUTHORIZED)
    }
}

impl PartialEq for ApiError {
    fn eq(&self, rhs: &Self) -> bool { self.response == rhs.response }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.response) }
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { Some(&self.source) }
}
