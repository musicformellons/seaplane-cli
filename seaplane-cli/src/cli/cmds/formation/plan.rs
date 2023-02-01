use clap::{ArgMatches, Command};
use const_format::concatcp;

#[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))]
use crate::cli::cmds::flight::SeaplaneFlightPlan;
use crate::{
    cli::{
        cmds::formation::{common, SeaplaneFormationFetch, SeaplaneFormationLaunch},
        specs::{FLIGHT_SPEC, REGION_SPEC},
        CliCommand,
    },
    context::{Ctx, FlightCtx},
    error::{CliErrorKind, Context, Result},
    ops::formation::{Formation, FormationConfiguration},
    printer::Color,
};

static LONG_ABOUT: &str =
    "Make a new local Formation Plan (and optionally launch an instance of it)

Include local Flight Plans by using `--include-flight-plan`. Multiple Flights may be included in a
Formation Plan using a SEMICOLON separated list, or using the argument multiple times.

You can also create a new Flight Plan using the INLINE-SPEC option of `--include-flight-plan`.

Flight Plans created using INLINE-SPEC are automatically included in the Formation Plan.";

/// A newtype wrapper to enforce where the ArgMatches came from which reduces errors in checking if
/// values of arguments were used or not. i.e. `seaplane formation plan` may not have the same
/// arguments as `seaplane account token` even though both produce an `ArgMatches`.
#[allow(missing_debug_implementations)]
pub struct SeaplaneFormationPlanArgMatches<'a>(pub &'a ArgMatches);

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationPlan;

impl SeaplaneFormationPlan {
    pub fn command() -> Command {
        Command::new("plan")
            .after_help(concatcp!(FLIGHT_SPEC, "\n\n", REGION_SPEC))
            .visible_aliases(["create", "add"])
            .about("Create a Seaplane Formation")
            .long_about(LONG_ABOUT)
            .args(common::args())
            .arg(arg!(--force).help("Override any existing Formation with the same NAME"))
            .arg(arg!(--fetch|sync|synchronize - ('F')).help("Fetch remote instances prior to creating this plan to check for conflicts (by default only local references are considered)"))
    }
}

impl CliCommand for SeaplaneFormationPlan {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        if ctx.args.fetch {
            let old_name = ctx.args.name_id.take();
            ctx.internal_run = true;
            SeaplaneFormationFetch.run(ctx)?;
            ctx.internal_run = false;
            ctx.args.name_id = old_name;
        }

        let formation_ctx = ctx.formation_ctx.get_or_init();

        // Check for duplicates and suggest `seaplane formation edit`
        let name = &formation_ctx.name_id;
        if ctx.db.formations.contains_name(name) {
            if !ctx.args.force {
                let mut err = CliErrorKind::DuplicateName(name.to_owned())
                    .into_err()
                    .context("(hint: try '")
                    .color_context(Color::Green, format!("seaplane formation edit {}", &name))
                    .context("' instead)\n");
                if ctx.db.needs_persist {
                    err = err.context("\nRolling back created Flight Plans!\n");
                }
                return Err(err);
            }

            // We have duplicates, but the user passed --force. So first we remove the existing
            // formations and "re-add" them

            // TODO: We should check if these ones we remove are referenced remote or not
            // TODO: if more than one formation has the exact same name, we remove them all; that's
            // *probably* what we want? But more thought should go into this...
            ctx.db.formations.remove_name(name);
        }

        if ctx.db.needs_persist {
            // Any flights we created in update_ctx can now be persisted. We didn't want to persist
            // them before as we could have still hit an error such as Duplicate Formation Names
            ctx.persist_flights()?;
            ctx.db.needs_persist = false;
        }

        // Add the new formation
        let mut new_formation = Formation::new(&formation_ctx.name_id);

        let cfg = formation_ctx.configuration_model(ctx)?;
        let formation_cfg = FormationConfiguration::new(cfg);
        new_formation.local.insert(formation_cfg.id);
        ctx.db.formations.configurations.push(formation_cfg);

        let id = new_formation.id.to_string();
        ctx.db.formations.formations.push(new_formation);

        ctx.persist_formations()?;

        cli_print!("Successfully created local Formation Plan '");
        cli_print!(@Green, "{}", &formation_ctx.name_id);
        cli_print!("' with ID '");
        cli_print!(@Green, "{}'\n", &id[..8]);

        // Equivalent of doing 'seaplane formation launch NAME --exact'
        if formation_ctx.launch || formation_ctx.grounded {
            // Set the name of the formation for launch
            ctx.args.name_id = Some(formation_ctx.name_id.clone());
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

        // Create any flights required
        let mut flights: Vec<_> = matches
            .get_many::<String>("include-flight-plan")
            .unwrap_or_default()
            .collect();

        // Flights declared with i.e. name=FOO,image=nginx:latest
        let inline_flights = vec_remove_if!(flights, |f: &str| f.contains('='));
        for flight in inline_flights {
            let mut cloned_ctx = ctx.clone();
            // We set stateless because we don't want the created flights to be persisted until
            // we're ready (i.e. we're sure this formation will be created)
            cloned_ctx.args.stateless = true;
            cloned_ctx.internal_run = true;
            cloned_ctx
                .flight_ctx
                .init(FlightCtx::from_inline_flight(flight, &ctx.registry)?);

            #[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))]
            {
                let flight_plan: Box<dyn CliCommand> = Box::new(SeaplaneFlightPlan);
                flight_plan.run(&mut cloned_ctx)?;
            }

            let name = cloned_ctx.flight_ctx.get_or_init().name_id.clone();
            // copy the newly created flight out of the cloned context into the "real" one
            #[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))]
            ctx.db
                .flights
                .add_flight(cloned_ctx.db.flights.remove_flight(&name, true).unwrap());

            // Store the newly created Flight name as if it was passed by name via
            // `--include-flight-plan FOO`
            ctx.formation_ctx
                .get_mut_or_init()
                .cfg_ctx
                .flights
                .push(name);

            ctx.db.needs_persist = true;
        }

        // Flights using @path or @-
        #[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests")))]
        for name in ctx
            .db
            .flights
            .add_from_at_strs(vec_remove_if!(flights, |f: &str| f.starts_with('@')))?
        {
            ctx.formation_ctx
                .get_mut_or_init()
                .cfg_ctx
                .flights
                .push(name);
            ctx.db.needs_persist = true;
        }

        // Any flights we created will be persisted during `run` as we could still hit an error
        // such as Duplicate Formation Names and would need to roll them back

        ctx.formation_ctx
            .get_mut_or_init()
            .update_from_formation_plan(
                &SeaplaneFormationPlanArgMatches(matches),
                &ctx.db.flights,
            )?;

        Ok(())
    }
}
