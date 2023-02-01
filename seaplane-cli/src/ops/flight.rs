use std::{
    fs,
    io::{self, Read, Write},
};

use seaplane::api::compute::v1::Flight as FlightModel;
use serde::{Deserialize, Serialize};
use tabwriter::TabWriter;

use crate::{
    context::{Ctx, FlightCtx},
    error::{CliError, CliErrorKind, Context, Result},
    fs::{FromDisk, ToDisk},
    ops::Id,
    printer::{Color, Output},
};

/// A wrapper round a Flight model
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Flight {
    pub id: Id,
    #[serde(flatten)]
    pub model: FlightModel,
}

impl Flight {
    pub fn new(model: FlightModel) -> Self { Self { id: Id::new(), model } }

    pub fn from_json(s: &str) -> Result<Flight> { serde_json::from_str(s).map_err(CliError::from) }

    pub fn starts_with(&self, s: &str) -> bool {
        self.id.to_string().starts_with(s) || self.model.name().starts_with(s)
    }

    /// Applies the non-default differences from `ctx`
    pub fn update_from(&mut self, ctx: &FlightCtx, keep_src_name: bool) -> Result<()> {
        let mut dest_builder = FlightModel::builder();

        // Name
        if keep_src_name {
            dest_builder = dest_builder.name(self.model.name());
        } else {
            dest_builder = dest_builder.name(&ctx.name_id);
        }

        if let Some(image) = ctx.image.clone() {
            dest_builder = dest_builder.image_reference(image);
        } else {
            dest_builder = dest_builder.image_reference(self.model.image().clone());
        }

        if ctx.minimum != 1 {
            dest_builder = dest_builder.minimum(ctx.minimum);
        } else {
            dest_builder = dest_builder.minimum(self.model.minimum());
        }

        if let Some(max) = ctx.maximum {
            dest_builder = dest_builder.maximum(max);
        } else if ctx.reset_maximum {
            dest_builder.clear_maximum();
        } else if let Some(max) = self.model.maximum() {
            dest_builder = dest_builder.maximum(max);
        }

        // Architecture
        for arch in ctx.architecture.iter().chain(self.model.architecture()) {
            dest_builder = dest_builder.add_architecture(*arch);
        }

        // API Permission
        #[cfg(feature = "unstable")]
        {
            let orig_api_perms = self.model.api_permission();
            let cli_api_perms = ctx.api_permission;
            match (orig_api_perms, cli_api_perms) {
                (true, false) => dest_builder = dest_builder.api_permission(false),
                (false, true) => dest_builder = dest_builder.api_permission(true),
                _ => (),
            }
        }

        self.model = dest_builder.build().expect("Failed to build Flight");
        Ok(())
    }

    /// Creates a Flight from either a JSON string in STDIN (@-) or a path pointed to by @path.
    fn from_at_str(flight: &str) -> Result<Self> {
        // First try to create for a @- (STDIN)
        if flight == "@-" {
            let mut buf = String::new();
            let stdin = io::stdin();
            let mut stdin_lock = stdin.lock();
            stdin_lock.read_to_string(&mut buf)?;

            // TODO: we need to check for and handle duplicates
            let new_flight = Flight::from_json(&buf)?;
            return Ok(new_flight);
        // next try to create if using @path
        } else if let Some(path) = flight.strip_prefix('@') {
            let new_flight = Flight::from_json(
                &fs::read_to_string(path)
                    .map_err(CliError::from)
                    .context("\n\tpath: ")
                    .with_color_context(|| (Color::Yellow, path))?,
            )?;
            return Ok(new_flight);
        }

        Err(CliErrorKind::InvalidCliValue(None, flight.into()).into_err())
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(transparent)]
pub struct Flights {
    inner: Vec<Flight>,
}

impl Flights {
    /// Takes strings in the form of @- or @path and creates then adds them to the DB. Only one @-
    /// may be used or an Error is returned.
    ///
    /// Returns a list of any created IDs
    pub fn add_from_at_strs<S>(&mut self, flights: Vec<S>) -> Result<Vec<String>>
    where
        S: AsRef<str>,
    {
        if flights.iter().filter(|f| f.as_ref() == "@-").count() > 1 {
            return Err(CliErrorKind::MultipleAtStdin.into_err());
        }
        let mut ret = Vec::new();

        for flight in flights {
            let new_flight = Flight::from_at_str(flight.as_ref())?;
            ret.push(new_flight.model.name().to_owned());
            self.inner.push(new_flight);
        }

        Ok(ret)
    }

    /// Removes any Flight definitions from the matching indices and returns a Vec of all removed
    /// Flights
    pub fn remove_indices(&mut self, indices: &[usize]) -> Vec<Flight> {
        // TODO: There is probably a much more performant way to remove a bunch of times from a Vec
        // but we're talking such a small number of items this should never matter.

        indices
            .iter()
            .enumerate()
            .map(|(i, idx)| self.inner.remove(idx - i))
            .collect()
    }

    /// Returns all indices of where the flight's Name or ID (as hex) begins with the `needle`
    pub fn indices_of_left_matches(&self, needle: &str) -> Vec<usize> {
        // TODO: Are there any names that also coincide with valid hex?
        self.inner
            .iter()
            .enumerate()
            .filter(|(_idx, flight)| flight.starts_with(needle))
            .map(|(idx, _flight)| idx)
            .collect()
    }

    /// Returns all indices of where the flight's Name or ID (as hex) is an exact match of `needle`
    pub fn indices_of_matches(&self, needle: &str) -> Vec<usize> {
        // TODO: Are there any names that also coincide with valid hex?
        self.inner
            .iter()
            .enumerate()
            .filter(|(_idx, flight)| {
                flight.id.to_string() == needle || flight.model.name() == needle
            })
            .map(|(idx, _flight)| idx)
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Flight> { self.inner.iter() }

    pub fn clone_flight(&mut self, src: &str, exact: bool) -> Result<Flight> {
        let src_flight = self.remove_flight(src, exact)?;
        let model = src_flight.model.clone();

        // Re add the source flight
        self.inner.push(src_flight);

        Ok(Flight::new(model))
    }

    /// Either updates a matching local flight, or creates a new one. Returns NEW flight names and
    /// IDs
    pub fn update_or_create_flight(&mut self, model: &FlightModel) -> Vec<(String, Id)> {
        let mut found = false;
        let mut ret = Vec::new();
        for flight in self
            .inner
            .iter_mut()
            .filter(|f| f.model.name() == model.name() && f.model.image_str() == model.image_str())
        {
            found = true;
            flight.model.set_minimum(model.minimum());
            flight.model.set_maximum(model.maximum());

            for arch in model.architecture() {
                flight.model.add_architecture(*arch);
            }

            #[cfg(feature = "unstable")]
            {
                flight.model.set_api_permission(model.api_permission());
            }
        }

        if !found {
            let f = Flight::new(model.clone());
            ret.push((f.model.name().to_owned(), f.id));
            self.inner.push(f);
        }

        ret
    }

    pub fn update_flight(&mut self, src: &str, exact: bool, ctx: &FlightCtx) -> Result<()> {
        let mut src_flight = self.remove_flight(src, exact)?;
        src_flight.update_from(ctx, ctx.generated_name)?;

        // Re add the source flight
        self.inner.push(src_flight);

        Ok(())
    }

    pub fn add_flight(&mut self, flight: Flight) { self.inner.push(flight); }

    pub fn remove_flight(&mut self, src: &str, exact: bool) -> Result<Flight> {
        // Ensure only one current Flight matches what the user gave
        // TODO: We should check if these ones we remove are referenced remote or not
        let indices =
            if exact { self.indices_of_matches(src) } else { self.indices_of_left_matches(src) };
        match indices.len() {
            0 => return Err(CliErrorKind::NoMatchingItem(src.into()).into_err()),
            1 => (),
            _ => return Err(CliErrorKind::AmbiguousItem(src.into()).into_err()),
        }

        Ok(self.remove_indices(&indices).pop().unwrap())
    }

    pub fn find_name(&self, name: &str) -> Option<&Flight> {
        self.inner.iter().find(|f| f.model.name() == name)
    }

    pub fn find_name_or_partial_id(&self, needle: &str) -> Option<&Flight> {
        self.inner
            .iter()
            .find(|f| f.model.name() == needle || f.id.to_string().starts_with(needle))
    }
}

// used in state v1 only
impl FromDisk for Flights {}
impl ToDisk for Flights {}

impl Output for Flights {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, ctx: &Ctx) -> Result<()> {
        let buf = Vec::new();
        let mut tw = TabWriter::new(buf);
        // TODO: Add local/remote formation references
        writeln!(tw, "LOCAL ID\tNAME\tIMAGE\tMIN\tMAX\tARCH\tAPI PERMS")?;
        for flight in self.iter() {
            let arch = flight
                .model
                .architecture()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");

            #[cfg_attr(not(feature = "unstable"), allow(unused_mut))]
            let mut api_perms = false;
            // Due to our use of cfg and Rust's "unused-assignment" lint
            let _ = api_perms;
            #[cfg(feature = "unstable")]
            {
                api_perms = flight.model.api_permission();
            }
            writeln!(
                tw,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                &flight.id.to_string()[..8], // TODO: make sure length is not ambiguous
                flight.model.name(),
                flight.model.image_str().trim_start_matches(&ctx.registry),
                flight.model.minimum(),
                flight
                    .model
                    .maximum()
                    .map(|n| format!("{n}"))
                    .unwrap_or_else(|| "INF".into()),
                if arch.is_empty() { "auto" } else { &*arch },
                api_perms,
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
