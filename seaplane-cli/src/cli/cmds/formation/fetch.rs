use clap::{ArgMatches, Command};

use crate::{
    api::FormationsReq,
    cli::{cmds::formation::common, errors, CliCommand},
    error::{CliErrorKind, Result},
    ops::formation::FormationNameId,
    printer::Pb,
    Ctx,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationFetch;

impl SeaplaneFormationFetch {
    pub fn command() -> Command {
        // TODO: add a --no-overwrite or similar
        Command::new("fetch-remote")
            .visible_aliases(["fetch", "sync", "synchronize"])
            .about("Fetch remote Formation Instances and create/synchronize local Plan definitions")
            .override_usage(
                "
    seaplane formation fetch-remote
    seaplane formation fetch-remote [NAME|ID]",
            )
            .arg(common::name_id(false).help(
                "The NAME or ID of the remote Formation Instance to fetch, omit to fetch all \
                Formation Instances",
            ))
    }
}

impl CliCommand for SeaplaneFormationFetch {
    // TODO: async
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let pb = Pb::new(ctx);
        let name_id = ctx.formation_ctx.get_or_init().name_id.clone();
        pb.set_message(format!(
            "Fetching Formation{}...",
            if name_id.is_some() { "s" } else { "" }
        ));

        let mut req = FormationsReq::new_delay_token(ctx)?;
        let names_ids: Vec<_> = if let Some(name_id) = &name_id {
            if name_id.is_oid() {
                req.set_id(*name_id.oid().unwrap())?;
            } else {
                req.set_id(
                    ctx.db
                        .formations
                        .get_by_name_id(name_id)
                        .ok_or_else(|| {
                            errors::no_matching_item(name_id.to_string(), false, false).unwrap_err()
                        })?
                        .model
                        .oid
                        .ok_or_else(|| {
                            CliErrorKind::OneOff(
                                "cannot fetch single Formation due to missing \
                            Formation ID, run 'seaplane formation fetch-remote' to fetch all \
                            Formations"
                                    .into(),
                            )
                            .into_err()
                        })?,
                )?;
            }
            let formation = req.get()?;

            // Note wrapping in a vec just to have the same branch type as above
            // This formation just came from the API so it has to have an OID
            let name_ids = vec![(formation.name.clone(), formation.oid.unwrap())];

            ctx.db.formations.create_or_update(formation);

            name_ids
        } else {
            let formations = req.get_all()?.objects;

            let name_ids = formations
                .iter()
                // This formation just came from the API so it has to have an OID
                .map(|f| (f.name.clone(), f.oid.unwrap()))
                .collect();

            for f in formations.into_iter() {
                ctx.db.formations.create_or_update(f);
            }

            name_ids
        };

        pb.finish_and_clear();

        if !ctx.internal_run {
            // Start going through the instances by formation
            for (name, oid) in names_ids.iter() {
                // prints:
                //   Successfully fetched Formation Instance foo (frm-abcdef12345)
                cli_print!("Successfully fetched Formation Instance ");
                cli_print!(@Green, "{name}");
                cli_print!(" (");
                cli_print!(@Green, "{oid}");
                cli_println!(")");
            }
        }

        ctx.persist_state()?;

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.name_id = matches.get_one::<FormationNameId>("name_id").cloned();
        Ok(())
    }
}
