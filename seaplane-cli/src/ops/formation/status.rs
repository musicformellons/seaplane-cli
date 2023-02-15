//! The status module is responsible for printing out a status page that looks like:
//!
//! ```not_rust
//! ◉ Formation stubb (frm-5wacbutjwbdexonddvdb2lnyxu): UNHEALTHY
//! └─┐
//!   │   FLIGHT    STATUS     OID
//!   ├─◉ flask     HEALTHY    flt-4bjyoaoqhbaorip3izfrou3siu
//!   └─◉ pequod    UNHEALTHY  flt-kr7dkiqwbrf35frwkm7vxsghci
//! ```
//!
//! Chars we'll need from (<http://www.unicode.org/charts/PDF/U2500.pdf>): `│ ├ ─ └ ┐`
use seaplane::api::compute::v2::{
    Flight as FlightModel, FlightId, FlightStatus as FlightStatusModel, FormationId,
};
use serde::Serialize;

use crate::{context::Ctx, error::Result, ops::formation::Formation, printer::Output};

// Possible Symbols?: ◯ ◉ ◍ ◐ ● ○ ◯
const SYM: char = '◉';

// Unfortunately we can't use tabwriter here as we can't color the symbol with that. So we
// just manually calculate the spaces since it's only a few fields anyways. We also assume
// the numbered fields aren't going to be higher than 99999999999 and if they are we most
// likely have other problems.
macro_rules! nspaces {
    ($n:expr, $w:expr) => {{
        nspaces!(($w.chars().count() + 4) - $n.to_string().len())
    }};
    ($n:expr) => {{
        let mut spaces = String::with_capacity($n);
        for _ in 0..$n {
            spaces.push(' ');
        }
        spaces
    }};
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct FormationStatus {
    name: String,
    oid: Option<FormationId>,
    flights: FlightStatuses,
    // Overall health of the Formation
    status: FlightStatusModel,
}

impl FormationStatus {
    pub fn update_status(&mut self) {
        let mut status = FlightStatusModel::Healthy;
        for flight in self.flights.inner.iter() {
            worse_only(&mut status, flight.status);
        }
        self.status = status;
    }
}

impl<'a> From<&'a Formation> for FormationStatus {
    fn from(value: &'a Formation) -> Self {
        let mut fs = FormationStatus {
            name: value.model.name.clone(),
            oid: value.model.oid,
            status: FlightStatusModel::Healthy,
            flights: FlightStatuses::with_capacity(value.model.flights.len()),
        };

        for flight in &value.model.flights {
            fs.flights.inner.push(flight.into());
        }

        fs.update_status();

        fs
    }
}

impl Output for FormationStatus {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, _ctx: &Ctx) -> Result<()> {
        print_sym(self.status);
        cli_print!(" Formation {}", self.name);
        if let Some(oid) = self.oid {
            cli_print!(" ({oid})");
        }
        cli_print!(": ");
        print_fsm(self.status);
        cli_println!("");
        if !self.flights.inner.is_empty() {
            cli_println!("└─┐");
            self.flights.print_pretty();
        }

        Ok(())
    }
}

impl Output for Vec<FormationStatus> {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, ctx: &Ctx) -> Result<()> {
        for fstatus in self.iter() {
            fstatus.print_table(ctx)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct FlightStatus {
    name: String,
    oid: Option<FlightId>,
    status: FlightStatusModel,
}

impl FlightStatus {
    pub fn print_pretty(&self, last: bool, flight_slot: usize, status_slot: usize) {
        if last {
            cli_print!("  └─");
        } else {
            cli_print!("  ├─");
        }
        print_sym(self.status);

        let name = &self.name;
        let status = self.status;

        let s_after_name = nspaces!(flight_slot - name.len());
        let s_after_status = nspaces!(status_slot - status_len(self.status));

        cli_print!(" {name}{s_after_name}");
        print_fsm(status);
        if let Some(oid) = self.oid {
            cli_println!("{s_after_status}{oid}");
        } else {
            cli_println!("");
        }
    }
}

impl<'a> From<&'a FlightModel> for FlightStatus {
    fn from(value: &'a FlightModel) -> Self {
        FlightStatus { name: value.name.clone(), oid: value.oid, status: value.status }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
struct FlightStatuses {
    inner: Vec<FlightStatus>,
}

impl FlightStatuses {
    pub fn with_capacity(capacity: usize) -> Self { Self { inner: Vec::with_capacity(capacity) } }

    pub fn print_pretty(&self) {
        if self.inner.is_empty() {
            return;
        }

        let longest_flight_name = self.inner.iter().map(|f| f.name.len()).max().unwrap();
        let total_flight_slot_size = std::cmp::max(longest_flight_name + 4, 10); // 10 = FLIGHT+4
        let spaces_after_flight = total_flight_slot_size - 6; // 6 = FLIGHT
        let longest_status = self
            .inner
            .iter()
            .map(|f| status_len(f.status))
            .max()
            .unwrap();
        let total_status_slot_size = std::cmp::max(longest_status + 4, 10); // 10 = STATUS+4
        let spaces_after_status = total_status_slot_size - 6; // 6 = STATUS
        cli_println!(
            "  │   FLIGHT{}STATUS{}OID",
            nspaces!(spaces_after_flight),
            nspaces!(spaces_after_status)
        );
        for (i, flight) in self.inner.iter().enumerate() {
            flight.print_pretty(
                i == self.inner.len() - 1,
                total_flight_slot_size,
                total_status_slot_size,
            );
        }
    }
}

pub fn worse_only(lhs: &mut FlightStatusModel, rhs: FlightStatusModel) {
    use FlightStatusModel::*;
    match lhs {
        Healthy => match rhs {
            Unhealthy => *lhs = Unhealthy,
            Starting => *lhs = Starting,
            _ => (),
        },
        Unhealthy => (),
        Starting => match rhs {
            Healthy => *lhs = Healthy,
            Unhealthy => *lhs = Unhealthy,
            _ => (),
        },
    }
}

/// Prints the SYM character color coded to the current status
pub fn print_sym(fsm: FlightStatusModel) {
    use FlightStatusModel::*;
    match fsm {
        Healthy => cli_print!(@Green, "{SYM}"),
        Unhealthy => cli_print!(@Red, "{SYM}"),
        Starting => cli_print!(@Yellow, "{SYM}"),
    }
}

/// Prints string version of self color coded to the current status
pub fn print_fsm(fsm: FlightStatusModel) {
    use FlightStatusModel::*;
    match fsm {
        Healthy => cli_print!(@Green, "Healthy"),
        Unhealthy => cli_print!(@Red, "Unhealthy"),
        Starting => cli_print!(@Yellow, "Starting"),
    }
}

fn status_len(f: FlightStatusModel) -> usize {
    use FlightStatusModel::*;
    match f {
        Healthy => 7,
        Unhealthy => 9,
        Starting => 8,
    }
}
