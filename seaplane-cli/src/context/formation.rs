use seaplane::api::compute::v2::Formation as FormationModel;

use crate::{
    context::flight::FlightCtx,
    error::{CliError, CliErrorKind, Context, Result},
    ops::{formation::FormationNameId, generate_name},
    printer::Color,
};

pub fn no_matching_flight(flight: &str) -> CliError {
    CliErrorKind::NoMatchingItem(flight.to_string())
        .into_err()
        .context("(hint: or try fetching remote definitions with '")
        .color_context(Color::Green, "seaplane formation fetch-remote")
        .context("')\n")
}

/// Represents the "Source of Truth" i.e. it combines all the CLI options, ENV vars, and config
/// values into a single structure that can be used later to build models for the API or local
/// structs for serializing
///
/// A somewhat counter-intuitive thing about Formations and their models is the there is no
/// "Formation Model" only a "Formation Configuration Model" This is because a "Formation" so to
/// speak is really just a named collection of configurations and info about their traffic
/// weights/activation statuses.
// TODO: we may not want to derive this we implement circular references
#[derive(Debug, Clone)]
pub struct FormationCtx {
    pub name_id: Option<FormationNameId>,
    pub launch: bool,
    pub remote: bool,
    pub local: bool,
    pub flights: Vec<FlightCtx>,
    pub gateway_flight: Option<String>,
    // Used internally to pass already gathered DB indices between operations
    pub indices: Option<Vec<usize>>,
}

impl Default for FormationCtx {
    fn default() -> Self {
        Self {
            name_id: None,
            launch: false,
            remote: false,
            local: true,
            flights: Vec::new(),
            indices: None,
            gateway_flight: None,
        }
    }
}

impl FormationCtx {
    /// Creates a new seaplane::api::compute::v1::FormationConfiguration from the contained values
    pub fn model(&self) -> Result<FormationModel> {
        // Create the new Formation model from the CLI inputs
        let mut f_model = FormationModel::builder();

        if let Some(FormationNameId::Name(name)) = &self.name_id {
            f_model = f_model.name(name);
        } else {
            f_model = f_model.name(generate_name());
        }

        for flight in &self.flights {
            f_model = f_model.add_flight(flight.model());
        }

        if let Some(gw) = &self.gateway_flight {
            f_model = f_model.gateway_flight(gw);
        }

        // TODO: probably match and check errors
        f_model.build().map_err(Into::into)
    }
}
