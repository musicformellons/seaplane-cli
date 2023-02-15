use clap::{ArgMatches, Command};

use crate::{
    cli::{
        cmds::formation::{common, SeaplaneFormationLand},
        CliCommand,
    },
    context::Ctx,
    error::Result,
    ops::formation::FormationNameId,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationDelete;

impl SeaplaneFormationDelete {
    pub fn command() -> Command {
        Command::new("delete")
            .visible_aliases(["del", "remove", "rm"])
            .about("Deletes local Formation Plans and/or remote Formation Instances")
            .override_usage("
    seaplane formation delete [OPTIONS] <NAME|ID>
    seaplane formation delete [OPTIONS] <NAME|ID> --no-remote")
            .arg(common::fetch(true))
            .arg(common::all())
            .arg(common::name_id(true)
                .help("The name or ID of the Formation to remove, must be unambiguous"))
            .arg(arg!(--force -('f'))
                .help("Delete this Formation even if there are remote instances without confirmation"))
            .arg(arg!(--local)
                .overrides_with("no-local")
                .help("Delete local Formation Definitions (this is set by the default, use --no-local to skip)"))
            .arg(arg!(--("no-local"))
                .overrides_with("local")
                .help("DO NOT delete local Formation Definitions"))
            .arg(arg!(--remote)
                .overrides_with("no-remote")
                .help("Delete remote Formation Instances (this is set by default, use --no-remote to skip)"))
            .arg(arg!(--("no-remote"))
                .overrides_with("remote")
                .help("DO NOT delete remote Formation Instances (this is set by the default, use --remote to remove them)"))
    }
}

impl CliCommand for SeaplaneFormationDelete {
    // TODO: add confirmation and skip confirmation with --force
    // TODO: add JSON format output
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let (local, remote) = {
            let fctx = ctx.formation_ctx.get_or_init();
            (fctx.local, fctx.remote)
        };
        if !(local || remote) {
            cli_eprint!(@Red, "error: ");
            cli_eprintln!("nothing to do");
            cli_eprint!("(hint: add ");
            cli_eprint!(@Yellow, "--local ");
            cli_eprint!("or ");
            cli_eprint!(@Yellow, "--remote ");
            cli_eprintln!("to the command)");
            std::process::exit(1);
        }

        // Remove the Formations
        //
        // First try to delete the remote formation if required, because we don't want to delete
        // the local one too if this fails
        if remote {
            SeaplaneFormationLand.run(ctx)?;
        }
        let indices = common::indices_matching_name_id(ctx)?;
        if local && !ctx.args.stateless {
            for formation in ctx.db.formations.remove_formation_indices(&indices).iter() {
                cli_print!("Deleted local Formation Plan ");
                cli_println!(@Green, "{}", formation.model.name);
            }
        }
        ctx.persist_state()?;

        if !indices.is_empty() {
            // TODO: recalculate dichotomy of local v. remote numbers (i.e. --no-local, etc.)
            let num_deleted = indices.len();
            cli_println!(
                "\nSuccessfully removed {} item{} from the local DB",
                num_deleted,
                if num_deleted > 1 { "s" } else { "" }
            );
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.force = matches.get_flag("force");
        ctx.args.all = matches.get_flag("all");
        ctx.args.fetch = matches.get_flag("fetch");
        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.name_id = matches.get_one::<FormationNameId>("name_id").cloned();
        fctx.remote = !matches.get_flag("no-remote");
        fctx.local = !matches.get_flag("no-local");

        Ok(())
    }
}
