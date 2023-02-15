mod status;

use std::io::Write;

use seaplane::api::compute::v2::{Formation as FormationModel, FormationId};
use serde::{Deserialize, Serialize};
use tabwriter::TabWriter;

use crate::{
    context::{Ctx, FormationCtx},
    error::{CliError, Result},
    ops::{formation::status::FormationStatus, NameId},
    printer::Output,
};

pub type FormationNameId = NameId<FormationId>;

/// This struct represents a Local Formation. I.e. one the user can interact with on the CLI and can
/// be (de)serialized locally.
///
/// A somewhat counter-intuitive thing about "Formations" and their models is the there is no
/// "Formation Model" only a "Formation Model" This is because a "Formation" so to
/// speak is really just a named collection of configurations and info about their traffic
/// weights/activation statuses.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(transparent)]
pub struct Formations {
    /// A list of "Formation"s
    pub inner: Vec<Formation>,
}

impl Formations {
    /// Extend and de-duplicate from another set of Formations
    pub fn extend(&mut self, rhs: &Formations) {
        self.inner.extend_from_slice(&rhs.inner);
        self.inner.dedup_by(|l, r| l.model.oid == r.model.oid);
    }

    pub fn oids_from_indices(&self, indices: &[usize]) -> Vec<FormationId> {
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(i, f)| if indices.contains(&i) { f.model.oid } else { None })
            .collect()
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Returns all indices of an exact name or partial ID match
    pub fn formation_indices_of_matches(&self, name_id: &FormationNameId) -> Vec<usize> {
        cli_traceln!("Searching local DB for exact matches of Formation Plan {name_id}");
        self.inner
            .iter()
            .enumerate()
            .filter(|(_, f)| match name_id {
                FormationNameId::Name(name) => &f.model.name == name,
                FormationNameId::Oid(oid) => f.model.oid == Some(*oid),
            })
            .map(|(i, _)| i)
            .collect()
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Returns all indices of a partial name or ID match
    ///
    /// The odd thing about how this works is due to FormationNameId. A partial OID is not a valid
    /// FormationId, thus it gets interpreted as a name. So in this partial search we only use the
    /// Name variant of the FormationNameId
    ///
    /// However, if a full OID is passed, we check it like normal
    pub fn formation_indices_of_partial_matches(&self, name_id: &FormationNameId) -> Vec<usize> {
        cli_traceln!("Searching local DB for partial matches of Formation Plan {name_id}");
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(i, f)| match name_id {
                FormationNameId::Name(name) => {
                    if f.model.name.contains(name)
                        || f.model.oid.map_or(false, |o| o.to_string().contains(name))
                    {
                        Some(i)
                    } else {
                        None
                    }
                }
                FormationNameId::Oid(oid) => {
                    if f.model.oid == Some(*oid) {
                        Some(i)
                    } else {
                        None
                    }
                }
            })
            .collect()
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Removes all indices
    pub fn remove_formation_indices(&mut self, indices: &[usize]) -> Vec<Formation> {
        cli_traceln!("Removing indexes {indices:?} from local state");
        // TODO: There is probably a much more performant way to remove a bunch of times from a Vec
        // but we're talking such a small number of items this should never matter.

        indices
            .iter()
            .enumerate()
            .map(|(i, idx)| self.inner.remove(idx - i))
            .collect()
    }

    pub fn get_by_name_id(&self, name_id: &FormationNameId) -> Option<&Formation> {
        self.inner.iter().find(|f| match name_id {
            FormationNameId::Name(name) => &f.model.name == name,
            FormationNameId::Oid(oid) => f.model.oid == Some(*oid),
        })
    }

    pub fn has_flight(&self, flight: &str) -> bool {
        self.inner
            .iter()
            .any(|f| f.model.flights.iter().any(|f| f.name == flight))
    }

    /// Either updates a matching local Formations, or creates a new one
    pub fn create_or_update(&mut self, formation: FormationModel) {
        if let Some(idx) = self.inner.iter().enumerate().find_map(|(i, f)| {
            if f.model.oid.is_some() && f.model.oid == formation.oid {
                Some(i)
            } else {
                None
            }
        }) {
            self.inner.swap_remove(idx);
            self.inner
                .push(Formation { model: formation, deployed: true, local: true });
        } else if let Some(idx) = self.inner.iter().enumerate().find_map(|(i, f)| {
            if f.model.name == formation.name {
                Some(i)
            } else {
                None
            }
        }) {
            self.inner.swap_remove(idx);
            self.inner
                .push(Formation { model: formation, deployed: true, local: true });
        }
    }

    /// We got a response from the API that has filled in all the OIDs for a Formation/Flights and
    /// URL so we update those here
    pub fn update(&mut self, model: &FormationModel) {
        if let Some(formation) = self.inner.iter_mut().find(|f| f.model.name == model.name) {
            // TODO: warn or error on existing?
            formation.model.oid = model.oid;
            formation.model.url = model.url.clone();
            for flight in formation.model.flights.iter_mut() {
                flight.oid = model
                    .flights
                    .iter()
                    .find(|fl| fl.name == flight.name)
                    .unwrap()
                    .oid;
            }
        } // TODO: warn or error on not found?
    }

    /// Returns true if there is a Formation with the given name or OID
    pub fn contains(&self, name_id: &FormationNameId) -> bool {
        self.get_by_name_id(name_id).is_some()
    }

    /// Removes an exact name match, returning the removed Formation or None if nothing matched.
    ///
    /// DANGER: this will invalidate any previously held indices after the removed item
    pub fn remove(&mut self, name_id: &FormationNameId) -> Option<Formation> {
        cli_traceln!("Removing Formation {name_id} from local state");
        if let Some(idx) = self.index_of(name_id) {
            return Some(self.inner.swap_remove(idx));
        }

        None
    }

    pub fn statuses(&self) -> Vec<FormationStatus> { self.inner.iter().map(Into::into).collect() }

    pub fn index_of(&self, name_id: &FormationNameId) -> Option<usize> {
        self.inner
            .iter()
            .enumerate()
            .find_map(|(i, f)| match name_id {
                FormationNameId::Name(name) => {
                    if &f.model.name == name {
                        Some(i)
                    } else {
                        None
                    }
                }
                FormationNameId::Oid(oid) => {
                    if f.model.oid == Some(*oid) {
                        Some(i)
                    } else {
                        None
                    }
                }
            })
    }

    pub fn add(&mut self, formation: Formation) { self.inner.push(formation); }

    pub fn iter(&self) -> impl Iterator<Item = &Formation> { self.inner.iter() }

    // TODO: this should go away once we're not working with indices anymore
    pub fn get(&self, idx: usize) -> Option<&Formation> { self.inner.get(idx) }

    // TODO: this should go away once we're not working with indices anymore
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Formation> { self.inner.get_mut(idx) }
}

impl Output for Formations {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, _ctx: &Ctx) -> Result<()> {
        let buf = Vec::new();
        let mut tw = TabWriter::new(buf);
        writeln!(tw, "NAME\tLOCAL\tDEPLOYED\tOID")?;
        for formation in &self.inner {
            writeln!(
                tw,
                "{}\t{:?}\t{:?}\t{}",
                formation.model.name,
                formation.local,
                formation.deployed,
                formation
                    .model
                    .oid
                    .map(|oid| oid.to_string())
                    .unwrap_or_else(String::new),
            )?;
        }
        tw.flush()?;

        cli_println!(
            "{}",
            String::from_utf8_lossy(
                &tw.into_inner()
                    .map_err(|_| CliError::bail("IO flush error"))?
            )
        );

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Formation {
    pub model: FormationModel,
    pub local: bool,
    pub deployed: bool,
}

impl<'a> From<&'a FormationCtx> for Formation {
    fn from(value: &'a FormationCtx) -> Self {
        let model = value.model().expect("FormationCtx invalid for Model");
        Formation { model: model.clone(), local: true, deployed: model.oid.is_some() }
    }
}
