use clap::{ArgMatches, Command};

use crate::{
    cli::{
        cmds::{
            flight::{
                common::{self, SeaplaneFlightCommonArgMatches},
                IMAGE_SPEC,
            },
            formation::SeaplaneFormationFetch,
        },
        CliCommand,
    },
    context::FlightCtx,
    error::{CliErrorKind, Context, Result},
    ops::flight::Flight,
    printer::Color,
    Ctx,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFlightPlan;

impl SeaplaneFlightPlan {
    pub fn command() -> Command {
        // TODO: add --from
        Command::new("plan")
            .visible_aliases(["create", "add"])
            .after_help(IMAGE_SPEC)
            .about("Make a new local Flight Plan that Formations can include and reference")
            .arg(arg!(--force - ('f')).help("Override any existing Flights Plans with the same NAME"))
            .arg(arg!(--fetch|sync|synchronize - ('F')).help("Fetch and synchronize remote Formation Instances (which reference Flight Plans) prior to creating this plan to check for conflicts (by default only local plans are checked)"))
            .args(common::args(true))
    }
}

impl CliCommand for SeaplaneFlightPlan {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        if ctx.args.stateless && !ctx.internal_run {
            cli_eprint!(@Red, "error: ");
            cli_eprint!("'");
            cli_eprint!(@Yellow, "--stateless");
            cli_eprint!("' cannot be used with '");
            cli_eprint!(@Yellow, "seaplane flight plan");
            cli_eprintln!("'");
            cli_eprintln!("(hint: 'seaplane flight plan' only modifies local plans)");
            cli_eprint!("(hint: you may want 'seaplane ");
            cli_eprint!(@Green, "formation ");
            cli_eprintln!("plan' instead)");
            std::process::exit(1);
        }

        if ctx.args.fetch {
            let old_name = ctx.args.name_id.take();
            SeaplaneFormationFetch.run(ctx)?;
            ctx.args.name_id = old_name;
        }

        let new_flight = ctx.flight_ctx.get_or_init().model();

        // Check for duplicates and suggest `seaplane flight edit`
        let name = new_flight.name();
        let indices = ctx.db.flights.indices_of_matches(name);
        if !indices.is_empty() {
            // TODO: We should check if these ones we remove are referenced remote or not

            if !ctx.args.force {
                return Err(CliErrorKind::DuplicateName(name.into())
                    .into_err()
                    .context("(hint: try '")
                    .color_context(Color::Green, format!("seaplane flight edit {name}"))
                    .context("' to edit an existing local Flight Plan)\n")
                    .context("(hint: you can also use '")
                    .color_context(Color::Green, "--force")
                    .context("' to overwrite existing items)\n"));
            }

            // We have duplicates, but the user passed --force. So first we remove the existing
            // Flights and "re-add" them

            // TODO: if more than one flight has the exact same name, we remove them all; that's
            // *probably* what we want? But more thought should go into this...
            ctx.db.flights.remove_indices(&indices);
        }

        let new_flight_name = new_flight.name().to_owned();
        // Add the new Flight
        let new_flight = Flight::new(new_flight);
        let id = new_flight.id;
        ctx.db.flights.add_flight(new_flight);

        ctx.persist_state()?;

        cli_print!("Successfully created Flight Plan '");
        cli_print!(@Green, "{new_flight_name}");
        cli_print!("' with ID '");
        cli_print!(@Green, "{}", &id.to_string()[..8]);
        cli_println!("'");

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.flight_ctx
            .init(FlightCtx::from_flight_common(&SeaplaneFlightCommonArgMatches(matches), ctx)?);
        ctx.args.force = matches.get_flag("force");
        ctx.args.fetch = matches.get_flag("fetch");
        Ok(())
    }
}
