#[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))]
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use seaplane::{
    api::compute::v2::Flight as FlightModel, rexports::container_image_ref::ImageReference,
};
use serde::Deserialize;

#[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))]
use crate::{error::Context, printer::Color};
use crate::{
    error::{CliError, CliErrorKind, Result},
    ops::{flight::str_to_image_ref, generate_name, validator::validate_name},
};

/// Represents the "Source of Truth" i.e. it combines all the CLI options, ENV vars, and config
/// values into a single structure that can be used later to build models for the API or local
/// structs for serializing
// TODO: we may not want to derive this we implement circular references
#[derive(Debug, Clone, Deserialize)]
pub struct FlightCtx {
    pub image: ImageReference,
    #[serde(rename = "name")]
    pub name_id: Option<String>,
    // True if we randomly generated the name. False if the user provided it
    #[serde(skip)]
    pub generated_name: bool,
}

impl FlightCtx {
    /// Builds a FlightCtx from a string value using the inline flight spec syntax:
    ///
    /// name=FOO,image=nginx:latest
    ///
    /// Where only image=... is required
    pub fn from_inline_flight(inline_flight: &str, registry: &str) -> Result<FlightCtx> {
        if inline_flight.contains(' ') {
            return Err(CliErrorKind::InlineFlightHasSpace.into_err());
        }

        let parts = inline_flight.split(',');

        macro_rules! parse_item {
            ($item:expr, $f:expr) => {{
                let mut item = $item.split('=');
                item.next();
                if let Some(value) = item.next() {
                    if value.is_empty() {
                        return Err(
                            CliErrorKind::InlineFlightMissingValue($item.to_string()).into_err()
                        );
                    }
                    $f(value)
                } else {
                    Err(CliErrorKind::InlineFlightMissingValue($item.to_string()).into_err())
                }
            }};
            ($item:expr) => {{
                parse_item!($item, |n| { Ok(n) })
            }};
        }

        let mut image = None;
        let mut generated_name = true;
        let mut name_id = None;

        for part in parts {
            match part.trim() {
                // @TODO technically nameFOOBAR=.. is valid... oh well
                name if part.starts_with("name") => {
                    name_id = parse_item!(name, |n: &str| {
                        if validate_name(n).is_err() {
                            Err(CliErrorKind::InlineFlightInvalidName(n.to_string()).into_err())
                        } else {
                            Ok(Some(n.to_string()))
                        }
                    })?;
                    generated_name = false;
                }
                // @TODO technically imageFOOBAR=.. is valid... oh well
                img if part.starts_with("image") => {
                    image = Some(str_to_image_ref(registry, parse_item!(img)?)?);
                }
                _ => {
                    return Err(CliErrorKind::InlineFlightUnknownItem(part.to_string()).into_err());
                }
            }
        }

        if image.is_none() {
            return Err(CliErrorKind::InlineFlightMissingImage.into_err());
        }
        if generated_name {
            name_id = Some(generate_name());
        }

        Ok(FlightCtx { image: image.unwrap(), name_id, generated_name })
    }

    /// Try to deserialize a Flight from a JSON string or convert to a CLI Error
    pub fn from_json(s: &str) -> Result<Self> { serde_json::from_str(s).map_err(CliError::from) }

    /// Create from an string which can be a PATH, `-` (STDIN), or the INLINE spec.
    pub fn from_str(flight: &str, registry: &str) -> Result<Vec<Self>> {
        cfg_if::cfg_if! {
            if #[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))] {
                // First try to create for a - (STDIN)
                if flight == "-" {
                    let mut buf = String::new();
                    let stdin = io::stdin();
                    let mut stdin_lock = stdin.lock();
                    stdin_lock.read_to_string(&mut buf)?;

                    return Ok(vec![FlightCtx::from_json(&buf)?]);
                }

                if flight.contains('=') {
                    return Ok(vec![FlightCtx::from_inline_flight(flight, registry)?]);
                }

                let mut res = Vec::new();
                for path in flight.split(',') {
                    if Path::exists(path.as_ref()) {
                        // next try to create if using path
                        res.push(FlightCtx::from_json(
                            &fs::read_to_string(path)
                                .map_err(CliError::from)
                                .context("\n\tpath: ")
                                .with_color_context(|| (Color::Yellow, path))?,
                        )?);
                    }
                }

                if res.is_empty() {
                    return Err(CliErrorKind::InvalidCliValue(None, flight.into()).into_err());
                }
                Ok(res)
            } else {
                // We're in a UI tests so just try to parse an inline spec and ignore everything
                // else
                if flight.contains('=') {
                    return Ok(vec![FlightCtx::from_inline_flight(flight, registry)?]);
                }
                Ok(Vec::new())
            }
        }
    }

    /// Creates a new seaplane::api::compute::v1::Flight from the contained values
    pub fn model(&self) -> FlightModel {
        // Create the new Flight model from the CLI inputs
        let flight_model = FlightModel::builder()
            .name(
                self.name_id
                    .clone()
                    .or_else(|| Some(generate_name()))
                    .unwrap(),
            )
            .image_reference(self.image.clone());

        // Create a new Flight struct we can add to our local JSON "DB"
        flight_model
            .build()
            .expect("Failed to build Flight from inputs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DEFAULT_IMAGE_REGISTRY_URL as IR;

    #[test]
    fn from_inline_flight_valid() {
        assert!(FlightCtx::from_inline_flight("image=demos/nginx:latest,name=foo", IR).is_ok());
        assert!(FlightCtx::from_inline_flight("image=demos/nginx:latest", IR).is_ok());
    }

    #[test]
    fn from_inline_flight_invalid() {
        assert_eq!(
            FlightCtx::from_inline_flight("image= demos/nginx:latest,name=foo", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightHasSpace
        );
        assert_eq!(
            FlightCtx::from_inline_flight("image=demos/nginx:latest, name=foo", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightHasSpace
        );
        assert_eq!(
            FlightCtx::from_inline_flight("name=foo", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightMissingImage
        );
        assert_eq!(
            FlightCtx::from_inline_flight(",image=demos/nginx:latest,name=foo", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightUnknownItem("".into())
        );
        assert_eq!(
            FlightCtx::from_inline_flight("image=demos/nginx:latest,", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightUnknownItem("".into())
        );
        assert_eq!(
            FlightCtx::from_inline_flight("image=demos/nginx:latest,foo", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightUnknownItem("foo".into())
        );
        assert_eq!(
            FlightCtx::from_inline_flight("image=demos/nginx:latest,name=invalid_name", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightInvalidName("invalid_name".into())
        );
        assert_eq!(
            FlightCtx::from_inline_flight("image=demos/nginx:latest,name", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightMissingValue("name".into())
        );
        assert_eq!(
            FlightCtx::from_inline_flight("image,name=foo", IR)
                .unwrap_err()
                .kind(),
            &CliErrorKind::InlineFlightMissingValue("image".into())
        );
    }
}
