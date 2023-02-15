use clap::{ArgMatches, Command};

use crate::{
    cli::{
        cmds::formation::{common, SeaplaneFormationLaunch},
        specs::FLIGHT_SPEC,
        validator::validate_path_inline,
        CliCommand,
    },
    context::{Ctx, FlightCtx},
    error::{CliErrorKind, Result},
    ops::{formation::FormationNameId, generate_name, validator::validate_name},
};

static LONG_ABOUT: &str =
    "Make a new local Formation Plan (and optionally launch an instance of it)

Include Flights by using `--flight`. Multiple Flights may be included in a Formation Plan using a
SEMICOLON separated list, or using the argument multiple times.";

static LONG_FLIGHT: &str =
    "A Flight to include in this Formation in the form of SPEC|path|- (See FLIGHT SPEC below)

Multiple items can be passed as a SEMICOLON (';') separated list or by using the argument multiple
times. Note that when using the SPEC it's usually easiest to only place one Flight per --flight
argument.

$ seaplane formation plan \\
    --flight name=flight1,image=nginx:latest \\
    --flight name=flight2,image=hello:latest

Which would include, two Flights (flight1, and flight2).";

static LONG_NAME: &str =
    "A human readable name for the Formation (must be unique within the tenant)

Rules for a valid name are as follows:

  - may only include ASCII lowercase, numbers and hyphens (0-9, a-z, and '-')
  - hyphens ('-') may not be repeated (i.e. '--')
  - no more than three (3) total hyphens
  - may not start or end with a hyphen
  - the total length must be <= 63

Some of these restrictions may be lifted in the future.";

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationPlan;

impl SeaplaneFormationPlan {
    pub fn command() -> Command {
        Command::new("plan")
            .after_help(FLIGHT_SPEC)
            .visible_aliases(["create", "add"])
            .about("Create a Seaplane Formation")
            .long_about(LONG_ABOUT)
            .arg(common::fetch(false))
            .arg(arg!(--name -('n') =["STRING"])
                .help("A friendly name for the Formation (unique within the tenant) if omitted a pseudo random name will be assigned. Note the name appears as part of the public Formation URL")
                .long_help(LONG_NAME)
                .value_parser(validate_name)
            )
            .arg(arg!(--launch).help("This Formation Plan should be deployed right away"))
            .arg(arg!(--flight|flights -('F') =["SPEC"]...)
                .help("Use Flight in this Formation in the form of SPEC|path|- (supports semicolon (';') separated list, or multiple uses) (See FLIGHT SPEC below)")
                .value_delimiter(';')
                .required(true)
                .long_help(LONG_FLIGHT)
                .value_parser(validate_path_inline)
            )
            .arg(arg!(--("gateway-flight") -('G') =["NAME"])
                .help("The name of the Flight to be used as the public traffic gateway that will receive all traffic that arrives on the public URL (if only a single Flight is included in this Formation, it will be implied as the gateway)")
            )
            .arg(arg!(--force).help("Override any existing Formation with the same NAME"))
    }
}

impl CliCommand for SeaplaneFormationPlan {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        common::run_fetch(ctx)?;

        let formation_ctx = ctx.formation_ctx.get_or_init();

        // TODO: eventually check for duplicates and suggest `seaplane formation edit`
        // name is either given or auto-generated so safe to unwrap
        let name = &formation_ctx.name_id.clone().unwrap();
        if ctx.db.formations.contains(name) {
            if !ctx.args.force {
                return Err(CliErrorKind::DuplicateName(name.to_string()).into_err());
            }

            // We have duplicates, but the user passed --force. So first we remove the existing
            // formations and "re-add" them

            // TODO: We should check if these ones we remove are referenced remote or not
            // TODO: if more than one formation has the exact same name, we remove them all; that's
            // *probably* what we want? But more thought should go into this...
            ctx.db.formations.remove(name);
        }

        ctx.db.formations.add(formation_ctx.into());

        ctx.persist_state()?;

        cli_print!("Successfully created local Formation Plan ");
        cli_println!(@Green, "{}", name);

        // Equivalent of doing 'seaplane formation launch NAME --exact'
        if formation_ctx.launch {
            // We only want to match this exact formation
            ctx.args.exact = true;
            // If `--fetch` was passed, we already did it, no need to do it again
            ctx.args.fetch = false;
            // release the MutexGuard
            SeaplaneFormationLaunch.run(ctx)?;
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.fetch = matches.get_flag("fetch");
        ctx.args.force = matches.get_flag("force");

        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.launch = matches.get_flag("launch");
        fctx.name_id = Some(FormationNameId::Name(
            matches
                .get_one::<String>("name")
                .map(ToOwned::to_owned)
                .unwrap_or_else(generate_name),
        ));

        for flight in matches.get_many::<String>("flight").unwrap_or_default() {
            fctx.flights
                .extend(FlightCtx::from_str(flight, &ctx.registry)?);
        }

        Ok(())
    }
}
