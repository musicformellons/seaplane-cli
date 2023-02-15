use clap::{ArgMatches, Command};
#[cfg(not(feature = "api_tests"))]
use seaplane::{api::ApiErrorKind, error::SeaplaneError};

use crate::{
    api::FormationsReq,
    cli::{cmds::formation::common, CliCommand},
    error::{CliErrorKind, Result},
    ops::formation::FormationNameId,
    Ctx,
};

static LONG_ABOUT: &str = "Land (Stop) all configurations of a remote Formation Instance

Unlike 'seaplane formation delete' the land command does not delete the Formation from the local
database.";

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationLand;

impl SeaplaneFormationLand {
    pub fn command() -> Command {
        Command::new("land")
            .visible_alias("stop")
            .about("Land a remote Formation Instance")
            .long_about(LONG_ABOUT)
            .arg(common::name_id(true).help("The name or ID of the Formation Instance to land"))
            .arg(common::all())
            .arg(common::fetch(true))
    }
}

impl CliCommand for SeaplaneFormationLand {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        common::run_fetch(ctx)?;

        let formation_ctx = ctx.formation_ctx.get_or_init();
        // If this is an internal run being called by Delete it will have already calculated the
        // indices for us
        let oids = if let Some(indices) = formation_ctx.indices.clone() {
            ctx.db.formations.oids_from_indices(&indices)
        } else {
            common::oids_matching_name_id(ctx)?
        };

        if oids.is_empty() {
            return Err(CliErrorKind::OneOff(
                "cannot land Formation due to missing Formation ID, run 'seaplane formation \
                fetch-remote' to synchronize the local database and try again"
                    .into(),
            )
            .into_err());
        }

        let mut req = FormationsReq::new_delay_token(ctx)?;
        for oid in &oids {
            req.set_id(*oid)?;
            #[cfg_attr(feature = "api_tests", allow(clippy::question_mark))]
            if let Err(e) = req.delete() {
                // Ignoring a 404 NOT FOUND during mock tests is bad
                #[cfg(not(feature = "api_tests"))]
                if matches!(
                    e.kind(),
                    CliErrorKind::Seaplane(SeaplaneError::ApiResponse(ae))
                    if ae.kind == ApiErrorKind::NotFound)
                {
                    // TODO: warn not found, only if --remote?
                    continue;
                }
                return Err(e);
            }
            // prints:
            //   Successfully Landed remote Formation Instance frm-abcdef12345 (stubb)
            cli_print!("Successfully Landed remote Formation Instance ");
            cli_print!(@Green, "{oid}");
            if let Some(f) = ctx
                .db
                .formations
                .get_by_name_id(&FormationNameId::Oid(*oid))
            {
                cli_print!(" (");
                cli_print!(@Green, "{}", f.model.name);
                cli_print!(")");
            }
            cli_println!("");
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.all = matches.get_flag("all");
        ctx.args.fetch = matches.get_flag("fetch");
        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.name_id = matches.get_one::<FormationNameId>("name_id").cloned();
        Ok(())
    }
}
