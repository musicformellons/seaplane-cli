use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use url::Url;

#[cfg(doc)]
use crate::api::compute::v2::FormationsRequest;
use crate::{
    api::compute::v2::validate_formation_name,
    error::{Result, SeaplaneError},
    rexports::{
        container_image_ref::ImageReference,
        seaplane_oid::{OidPrefix, TypedOid},
    },
};

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Frm;
impl OidPrefix for Frm {}

/// A Formation Object ID, ex. `frm-agc6amh7z527vijkv2cutplwaa`
pub type FormationId = TypedOid<Frm>;

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Flt;
impl OidPrefix for Flt {}

/// A Flight Object ID, ex. `flt-agc6amh7z527vijkv2cutplwaa`
pub type FlightId = TypedOid<Flt>;

/// Whether a Flight is Health or Unhealthy as determined by the runtime
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, Display, Default)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum FlightStatus {
    /// The Flight is healthy
    Healthy,

    /// The Flight is unhealthy
    Unhealthy,

    /// The Flight is starting and has not yet reported a health status
    #[default]
    Starting,
}

impl FlightStatus {
    pub fn is_starting(&self) -> bool { self == &FlightStatus::Starting }
}

impl_serde_str!(FlightStatus);

#[cfg(test)]
mod flight_health_status_tests {
    use super::*;

    #[test]
    fn deser() {
        assert_eq!(FlightStatus::Healthy, "healthy".parse().unwrap());
        assert_eq!(FlightStatus::Healthy, "Healthy".parse().unwrap());
        assert_eq!(FlightStatus::Healthy, "HEALTHY".parse().unwrap());

        assert_eq!(FlightStatus::Unhealthy, "unhealthy".parse().unwrap());
        assert_eq!(FlightStatus::Unhealthy, "Unhealthy".parse().unwrap());
        assert_eq!(FlightStatus::Unhealthy, "UNHEALTHY".parse().unwrap());

        assert_eq!(FlightStatus::Starting, "starting".parse().unwrap());
        assert_eq!(FlightStatus::Starting, "Starting".parse().unwrap());
        assert_eq!(FlightStatus::Starting, "STARTING".parse().unwrap());
    }

    #[test]
    fn ser() {
        assert_eq!(FlightStatus::Healthy.to_string(), "healthy".to_string());
        assert_eq!(FlightStatus::Unhealthy.to_string(), "unhealthy".to_string());
        assert_eq!(FlightStatus::Starting.to_string(), "starting".to_string());
    }
}

/// A builder for creating a [`Formation`] which is the primary way to describe a
/// valid configuration for a Formation.
#[derive(Debug, Default)]
pub struct FormationBuilder {
    flights: Vec<Flight>,
    name: String,
    gateway_flight: Option<String>,
}

impl FormationBuilder {
    /// The human readable [`Formation`] name, which must be unique within the Formation and URL
    /// safe. See [`validate_formation_name`] for more information.
    ///
    /// **NOTE:** The name will be validated on the call to [`FormationBuilder::build`]
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add a [`Flight`] to the makeup of this Formation Configuration.
    ///
    /// **NOTE:** This method can be called multiple times. All values will be utilized.
    #[must_use]
    pub fn add_flight(mut self, flight: Flight) -> Self {
        self.flights.push(flight);
        self
    }

    pub fn gateway_flight(mut self, flight: impl Into<String>) -> Self {
        // @TODO validate flight name
        self.gateway_flight = Some(flight.into());
        self
    }

    /// Removes all [`Flight`]s from this Formation Configuration
    pub fn clear_flights(&mut self) { self.flights.clear(); }

    /// Performs validation checks, and builds the instance of [`Formation`]
    pub fn build(self) -> Result<Formation> {
        if self.flights.is_empty() {
            return Err(SeaplaneError::EmptyFlights);
        }

        // Ensure gateway_flight was defined
        if self
            .gateway_flight
            .as_ref()
            .map(|gw_f| self.flights.iter().any(|f| &f.name == gw_f))
            == Some(false)
        {
            return Err(SeaplaneError::InvalidGatewayFlight);
        }

        validate_formation_name(&self.name)?;

        Ok(Formation {
            name: self.name,
            oid: None,
            url: None,
            flights: self.flights,
            gateway_flight: self.gateway_flight,
        })
    }
}

/// Represents a single Formation.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Formation {
    /// The human friendly name of the Formation
    pub name: String,

    /// The Object ID of the Formation that will be assigned by the Compute API upon launch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oid: Option<FormationId>,

    /// The public URL this Formation is exposed on
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,

    /// The Flights that make up this Formation
    pub flights: Vec<Flight>,

    /// The Flight who will receive all the public HTTP(s) traffic that arrives on the public
    /// Formation URL
    pub gateway_flight: Option<String>,
}

impl Formation {
    /// Create a [`FormationBuilder`] to build a new configuration
    pub fn builder() -> FormationBuilder { FormationBuilder::default() }

    /// Add a [`Flight`] to the makeup of this Formation Configuration.
    pub fn add_flight(&mut self, flight: Flight) { self.flights.push(flight); }

    /// Remove a [`Flight`] from the makeup of this Formation Configuration.
    pub fn remove_flight(&mut self, name: &str) -> Option<Flight> {
        if let Some(i) =
            self.flights
                .iter()
                .enumerate()
                .find_map(|(i, f)| if f.name == name { Some(i) } else { None })
        {
            Some(self.flights.swap_remove(i))
        } else {
            None
        }
    }

    /// Set the [`Flight`]s that makeup this Formation Configuration.
    pub fn set_flights(&mut self, flights: Vec<Flight>) { self.flights = flights; }

    /// Set the [`Flight`]s that makeup this Formation Configuration.
    pub fn flights(&self) -> &[Flight] { &self.flights }
}

#[cfg(test)]
mod formation_tests {
    use super::*;

    #[test]
    fn deser() {
        let json = r#"{
            "name": "example-formation",
            "oid": "frm-agc6amh7z527vijkv2cutplwaa",
            "url": "https://example-formation.tenant.on.cplane.cloud",
            "flights": [{
                "name":"example-flight",
                "oid":"flt-agc6amh7z527vijkv2cutplwaa",
                "image":"foo.com/bar:latest"
            }],
            "gateway-flight": "example-flight"
        }"#;
        let model = Formation {
            name: "example-formation".into(),
            oid: Some("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap()),
            url: Some(
                "https://example-formation.tenant.on.cplane.cloud"
                    .parse()
                    .unwrap(),
            ),
            flights: vec![Flight {
                name: "example-flight".into(),
                oid: Some("flt-agc6amh7z527vijkv2cutplwaa".parse().unwrap()),
                image: "foo.com/bar:latest".parse::<ImageReference>().unwrap(),
                status: FlightStatus::Starting,
            }],
            gateway_flight: Some("example-flight".into()),
        };

        assert_eq!(model, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn ser() {
        let json = r#"{"name":"example-formation","oid":"frm-agc6amh7z527vijkv2cutplwaa","flights":[{"name":"example-flight","oid":"flt-agc6amh7z527vijkv2cutplwaa","image":"foo.com/bar:latest"}],"gateway-flight":"example-flight"}"#;
        let model = Formation {
            name: "example-formation".into(),
            oid: Some("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap()),
            url: None,
            flights: vec![Flight {
                name: "example-flight".into(),
                oid: Some("flt-agc6amh7z527vijkv2cutplwaa".parse().unwrap()),
                image: "foo.com/bar:latest".parse::<ImageReference>().unwrap(),
                status: FlightStatus::Starting,
            }],
            gateway_flight: Some("example-flight".into()),
        };

        assert_eq!(json.to_string(), serde_json::to_string(&model).unwrap());
    }

    #[test]
    fn ser_no_oid() {
        let json = r#"{"name":"example-formation","flights":[{"name":"example-flight","image":"foo.com/bar:latest","status":"healthy"}],"gateway-flight":"example-flight"}"#;
        let model = Formation {
            name: "example-formation".into(),
            url: None,
            oid: None,
            flights: vec![Flight {
                name: "example-flight".into(),
                oid: None,
                image: "foo.com/bar:latest".parse::<ImageReference>().unwrap(),
                status: FlightStatus::Healthy,
            }],
            gateway_flight: Some("example-flight".into()),
        };

        assert_eq!(json.to_string(), serde_json::to_string(&model).unwrap());
    }
}

/// A builder to construct [`Flight`]s
#[derive(Debug, Default)]
pub struct FlightBuilder {
    name: Option<String>,
    image: Option<ImageReference>,
}

impl FlightBuilder {
    /// Create a new builder
    pub fn new() -> Self { Self::default() }

    /// The human readable [`Flight`] name, which must be unique within the Formation
    #[must_use]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// A container image registry reference which points to the container image this [`Flight`]
    /// should uses
    ///
    /// # Panics
    ///
    /// This method `panic!`s if the `image_ref` provided cannot be parsed into a valid
    /// [`ImageReference`]
    #[must_use]
    pub fn image<R: AsRef<str>>(mut self, image_ref: R) -> Self {
        self.image = Some(
            image_ref
                .as_ref()
                .parse::<ImageReference>()
                .expect("Failed to parse image reference"),
        );
        self
    }

    /// A container image registry reference which points to the container image this [`Flight`]
    /// should uses.
    ///
    /// This method allows providing a pre-parsed [`ImageReference`] instead of a string which can
    /// `panic!` on parsing in [`FlightBuilder::image`].
    #[must_use]
    pub fn image_reference(mut self, image_ref: ImageReference) -> Self {
        self.image = Some(image_ref);
        self
    }

    /// Perform validation checks and construct a [`Flight`]
    pub fn build(self) -> Result<Flight> {
        if self.name.is_none() {
            return Err(SeaplaneError::MissingFlightName);
        } else if self.image.is_none() {
            return Err(SeaplaneError::MissingFlightImageReference);
        }

        Ok(Flight {
            name: self.name.unwrap(),
            oid: None,
            image: self.image.unwrap(),
            status: FlightStatus::default(),
        })
    }
}

/// Describes a single [`Flight`] within a Formation.
///
/// Flights are logically a single container. However, Seaplane spins up many actual backing
/// *container instances* around the globe (with your Formation's `regions_allowed` map) and load
/// balances traffic between them.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct Flight {
    /// Returns the human readable name of the [`Flight`], which is unique with a Formation
    pub name: String,

    /// The Object ID of the Flight
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oid: Option<FlightId>,

    /// The container image reference
    pub image: ImageReference,

    /// The status of this Flight
    #[serde(default, skip_serializing_if = "FlightStatus::is_starting")]
    pub status: FlightStatus,
}

impl Flight {
    /// Create a new [`FlightBuilder`] in order to construct a new [`Flight`]
    pub fn builder() -> FlightBuilder { FlightBuilder::new() }

    /// Creates a new [`Flight`] with the two required bits of information, a `name` which must be
    /// unique within the Formation, and a container image registry URL which points to the
    /// container image to use.
    ///
    /// # Panics
    ///
    /// This method `panic!`s if the `image_ref` provided cannot be parsed into a valid
    /// [`ImageReference`]
    pub fn new<S, R>(name: S, image_ref: R) -> Flight
    where
        S: Into<String>,
        R: AsRef<str>,
    {
        FlightBuilder::new()
            .name(name)
            .image(image_ref)
            .build()
            .unwrap()
    }

    /// Returns the human readable [`Flight`] name, which is unique within the Formation
    #[inline]
    pub fn name(&self) -> &str { &self.name }

    /// Returns the container image reference this [`Flight`] uses, as a [`String`]
    #[inline]
    pub fn image_str(&self) -> String { self.image.to_string() }

    /// Returns the container image reference this [`Flight`] uses, as an [`ImageReference`]
    #[inline]
    pub fn image(&self) -> &ImageReference { &self.image }
}

#[cfg(test)]
mod flight_tests {
    use super::*;

    #[test]
    fn deser() {
        let json = r#"{
            "name":"example-flight",
            "oid":"flt-agc6amh7z527vijkv2cutplwaa",
            "image":"foo.com/bar:latest"
        }"#;
        let model = Flight {
            name: "example-flight".into(),
            oid: Some("flt-agc6amh7z527vijkv2cutplwaa".parse().unwrap()),
            image: "foo.com/bar:latest".parse::<ImageReference>().unwrap(),
            status: FlightStatus::Starting,
        };

        assert_eq!(model, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn ser() {
        let json = r#"{"name":"example-flight","oid":"flt-agc6amh7z527vijkv2cutplwaa","image":"foo.com/bar:latest","status":"healthy"}"#;
        let model = Flight {
            name: "example-flight".into(),
            oid: Some("flt-agc6amh7z527vijkv2cutplwaa".parse().unwrap()),
            image: "foo.com/bar:latest".parse::<ImageReference>().unwrap(),
            status: FlightStatus::Healthy,
        };

        assert_eq!(json, serde_json::to_string(&model).unwrap());
    }

    #[test]
    fn ser_no_oid() {
        let json = r#"{"name":"example-flight","image":"foo.com/bar:latest","status":"healthy"}"#;
        let model = Flight {
            name: "example-flight".into(),
            oid: None,
            image: "foo.com/bar:latest".parse::<ImageReference>().unwrap(),
            status: FlightStatus::Healthy,
        };

        assert_eq!(json, serde_json::to_string(&model).unwrap());
    }
}
