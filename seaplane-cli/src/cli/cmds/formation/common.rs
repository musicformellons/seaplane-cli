//! The common args effectively represent a "Formation" which many commands can
//! include.
//!
//! The only additional information is the formation name, which is not part of the configuration,
//! but many commands need as well.

use clap::Arg;
use seaplane::api::compute::v2::FormationId;

use crate::{
    cli::{cmds::formation::SeaplaneFormationFetch, errors, CliCommand},
    context::Ctx,
    error::{CliErrorKind, Result},
    ops::formation::FormationNameId,
};

pub fn name_id(required: bool) -> Arg {
    arg!(name_id = ["NAME|ID"])
        .value_parser(clap::value_parser!(FormationNameId))
        .required(required)
}

pub fn fetch(short: bool) -> Arg {
    let arg = arg!(--fetch | sync | synchronize).help(
        "Fetch remote Formation Instances and synchronize local DB prior to running this command",
    );
    if short {
        return arg.short('F');
    }
    arg
}

pub fn all() -> Arg {
    arg!(--all - ('a'))
        .help("Operate on all matching local Formation Plans even when the name or ID is ambiguous")
}

pub fn oids_matching_name_id(ctx: &Ctx) -> Result<Vec<FormationId>> {
    let formation_ctx = ctx.formation_ctx.get_or_init();
    let name_id = formation_ctx.name_id.as_ref().unwrap();
    match indices_matching_name_id(ctx) {
        Ok(indices) => {
            if indices.is_empty() {
                if let FormationNameId::Oid(oid) = name_id {
                    Ok(vec![*oid])
                } else {
                    unreachable!("this is a bug");
                }
            } else {
                Ok(ctx.db.formations.oids_from_indices(&indices))
            }
        }
        Err(e) => {
            if matches!(e.kind(), CliErrorKind::NoMatchingItem(_)) {
                if let FormationNameId::Oid(oid) = name_id {
                    return Ok(vec![*oid]);
                }
            }
            Err(e)
        }
    }
}

pub fn indices_matching_name_id(ctx: &Ctx) -> Result<Vec<usize>> {
    let formation_ctx = ctx.formation_ctx.get_or_init();
    let name_id = formation_ctx.name_id.as_ref().unwrap();

    // Get the indices of any formations that match the given name/ID
    let indices = if ctx.args.all {
        ctx.db
            .formations
            .formation_indices_of_partial_matches(name_id)
    } else {
        ctx.db.formations.formation_indices_of_matches(name_id)
    };

    match indices.len() {
        0 => {
            if name_id.is_oid() {
                // This case happens if we looked for an Oid but didn't find any in the local DB.
                // This is common if we haven't sync'ed or launched the formation yet because the
                // API hasn't yet given us an OID.
                //
                // So we can't match to an index/formation but we also shouldn't necessarily error,
                // we leave that error case up to the calling code to decide if failing to find a
                // match is an error
                return Ok(Vec::new());
            } else {
                errors::no_matching_item(name_id.to_string(), false, ctx.args.all)?;
            }
        }
        1 => (),
        _ => {
            if !(ctx.args.all || ctx.args.force) {
                errors::ambiguous_item(name_id.to_string(), true)?;
            }
        }
    }

    Ok(indices)
}

pub fn run_fetch(ctx: &mut Ctx) -> Result<()> {
    if ctx.args.fetch {
        let old_name = { ctx.formation_ctx.get_mut_or_init().name_id.take() };
        let old_ir = ctx.internal_run;
        ctx.internal_run = true;
        SeaplaneFormationFetch.run(ctx)?;
        ctx.internal_run = old_ir;
        ctx.formation_ctx.get_mut_or_init().name_id = old_name;
    }
    Ok(())
}
