use clap::{ArgMatches, Command};

use crate::{
    api::FormationsReq,
    cli::{cmds::formation::common, CliCommand},
    context::Ctx,
    error::Result,
    ops::{formation::FormationNameId, validator::validate_name},
    printer::Pb,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationLaunch;

impl SeaplaneFormationLaunch {
    pub fn command() -> Command {
        Command::new("launch")
            .visible_alias("start")
            .about("Start a local Formation Plan creating a remote Formation Instance")
            .arg(common::all())
            .arg(
                arg!(name = ["NAME"] required)
                    .value_parser(validate_name)
                    .help("The name of the Formation Plan to launch and create an Instance of"),
            )
    }
}

impl CliCommand for SeaplaneFormationLaunch {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let formation_ctx = ctx.formation_ctx.get_or_init();
        // If this is an internal run being called by Delete it will have already calculated the
        // indices for us
        let indices = if let Some(indices) = formation_ctx.indices.clone() {
            indices
        } else {
            common::indices_matching_name_id(ctx)?
        };

        let pb = Pb::new(ctx);
        let mut req = FormationsReq::new_delay_token(ctx)?;
        let mut formations_created = Vec::with_capacity(indices.len());
        for idx in indices {
            let formation = ctx.db.formations.get(idx).expect("invalid index");
            pb.set_message(format!("Launching Formation {}...", formation.model.name));

            let resp = req.create(&formation.model)?;
            // update our local DB with all the new OIDs we just got back from the server
            ctx.db.formations.update(&resp);
            // TODO: in the future it's possible not all Formations will have a public URL
            formations_created.push((resp.name, resp.oid.unwrap(), resp.url.unwrap()));
        }

        pb.finish_and_clear();
        for (name, oid, url) in formations_created {
            cli_print!("Successfully Launched remote Formation Instance ");
            cli_print!(@Green, "{name}");
            cli_print!(" (");
            cli_print!(@Green, "{oid}");
            cli_println!(")");

            cli_print!("Formation Instance URL is: ");
            cli_println!(@Green, "{url}");
            cli_println!(
                "(hint: it may take up to a minute for the Formation to become fully online)"
            );
            cli_print!("(hint: check the status of this Formation Instance with '");
            cli_print!(@Green, "seaplane formation status {name}");
            cli_println!("')");
        }
        ctx.persist_state()?;

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.all = matches.get_flag("all");
        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.name_id =
            Some(FormationNameId::Name(matches.get_one::<String>("name").cloned().unwrap()));
        Ok(())
    }
}
