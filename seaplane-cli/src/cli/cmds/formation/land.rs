use clap::{ArgMatches, Command};

use crate::{
    api::FormationsReq,
    cli::{
        cmds::formation::SeaplaneFormationFetch,
        errors,
        validator::{validate_formation_name, validate_name_id},
        CliCommand,
    },
    error::Result,
    Ctx,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationLand;

impl SeaplaneFormationLand {
    pub fn command() -> Command {
        let validator = |s: &str| validate_name_id(validate_formation_name, s);
        Command::new("land")
            .visible_alias("stop")
            .about("Land (Stop) all configurations of a remote Formation Instance")
            .arg(
                arg!(name_id =["NAME|ID"] required)
                    .help("The name or ID of the Formation Instance to land")
                    .value_parser(validator),
            )
            .arg(
                arg!(--all - ('a'))
                    .help("Stop all matching Formations even when FORMATION is ambiguous"),
            )
            .arg(arg!(--fetch|sync|synchronize - ('F')).help(
                "Fetch remote Formation Instances and synchronize local Plan definitions prior to attempting to land",
            ))
    }
}

impl CliCommand for SeaplaneFormationLand {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        if ctx.args.fetch {
            let old_name = ctx.args.name_id.take();
            ctx.internal_run = true;
            SeaplaneFormationFetch.run(ctx)?;
            ctx.internal_run = false;
            ctx.args.name_id = old_name;
        }
        let name = ctx.args.name_id.as_ref().unwrap();
        // Get the indices of any formations that match the given name/ID
        let indices = if ctx.args.all {
            ctx.db.formations.formation_indices_of_left_matches(name)
        } else {
            ctx.db.formations.formation_indices_of_matches(name)
        };

        match indices.len() {
            0 => errors::no_matching_item(name.to_string(), false, ctx.args.all)?,
            1 => (),
            _ => {
                if !ctx.args.all {
                    errors::ambiguous_item(name.to_string(), true)?;
                }
            }
        }

        let mut req = FormationsReq::new_delay_token(ctx)?;
        for idx in indices {
            // re unwrap: the indices returned came from Formations so they have to be valid
            let formation = ctx.db.formations.get_formation_mut(idx).unwrap();
            req.set_name(formation.name.as_ref().unwrap())?;

            // re unwrap: We got the formation from the local DB so it has to have a name
            req.stop()?;

            // Move all configurations from in air to grounded
            let ids: Vec<_> = formation.in_air.drain().collect();
            for id in ids {
                formation.grounded.insert(id);
            }

            ctx.persist_state()?;

            cli_print!("Successfully Landed remote Formation Instance '");
            cli_print!(@Green, "{name}");
            cli_println!("'");
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.all = matches.get_flag("all");
        ctx.args.name_id = matches.get_one::<String>("name_id").map(ToOwned::to_owned);
        ctx.args.fetch = matches.get_flag("fetch");
        Ok(())
    }
}
