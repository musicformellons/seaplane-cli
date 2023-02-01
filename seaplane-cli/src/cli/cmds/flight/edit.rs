use clap::{ArgMatches, Command};

use crate::{
    cli::{
        cmds::flight::{
            common::{self, SeaplaneFlightCommonArgMatches},
            IMAGE_SPEC,
        },
        errors::wrap_cli_context,
        validator::{validate_flight_name, validate_name_id},
        CliCommand,
    },
    context::{Ctx, FlightCtx},
    error::Result,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFlightEdit;

impl SeaplaneFlightEdit {
    pub fn command() -> Command {
        let validator = |s: &str| validate_name_id(validate_flight_name, s);
        // TODO: add --no-maximum or similar
        // TODO: add --from
        Command::new("edit")
            .about("Edit a local Flight Plan")
            .after_help(IMAGE_SPEC)
            .arg(
                arg!(name_id required =["NAME|ID"])
                    .help("The source name or ID of the Flight Plan to edit")
                    .value_parser(validator),
            )
            .arg(arg!(--exact - ('x')).help("The given name or ID must be an exact match"))
            .args(common::args(false))
    }
}

impl CliCommand for SeaplaneFlightEdit {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        if ctx.args.stateless {
            cli_eprint!(@Red, "error: ");
            cli_eprint!("'");
            cli_eprint!(@Yellow, "--stateless");
            cli_eprint!("' cannot be used with '");
            cli_eprint!(@Yellow, "seaplane flight edit");
            cli_eprintln!("'");
            cli_eprintln!("(hint: 'seaplane flight edit' only modifies local plans)");
            cli_eprint!("(hint: you may want 'seaplane ");
            cli_eprint!(@Green, "formation ");
            cli_eprintln!("edit' instead)");
            std::process::exit(1);
        }
        let flights = &mut ctx.db.flights;

        // Now we just edit the newly copied Flight to match the given CLI params...
        // name_id cannot be None in `flight edit`
        if let Err(e) = flights.update_flight(
            ctx.args.name_id.as_ref().unwrap(),
            ctx.args.exact,
            ctx.flight_ctx.get_or_init(),
        ) {
            return wrap_cli_context(e, ctx.args.exact, false);
        }

        ctx.persist_state()?;

        cli_print!("Successfully edited Flight Plan '");
        cli_print!(@Yellow, "{}", ctx.args.name_id.as_ref().unwrap());
        cli_println!("'");

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        // clap will not let "source" be None
        ctx.args.name_id = matches.get_one::<String>("name_id").map(ToOwned::to_owned);
        ctx.flight_ctx
            .init(FlightCtx::from_flight_common(&SeaplaneFlightCommonArgMatches(matches), ctx)?);
        Ok(())
    }
}
