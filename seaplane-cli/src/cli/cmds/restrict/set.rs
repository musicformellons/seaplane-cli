use clap::{ArgMatches, Command};
use serde_json::json;

use crate::{
    api::RestrictReq,
    cli::{cmds::restrict::common, specs::REGION_SPEC, CliCommand},
    context::{Ctx, RestrictCtx},
    error::Result,
    ops::EncodedString,
    printer::OutputFormat,
};

/// A newtype wrapper to enforce where the ArgMatches came from which reduces errors in checking if
/// values of arguments were used or not. i.e. `seaplane formation create` may not have the same
/// arguments as `seaplane account token` even though both produce an `ArgMatches`.
#[allow(missing_debug_implementations)]
pub struct SeaplaneRestrictSetArgMatches<'a>(pub &'a ArgMatches);

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneRestrictSet;

impl SeaplaneRestrictSet {
    pub fn command() -> Command {
        Command::new("set")
            .visible_alias("put")
            .about("Set a restriction")
            .arg(common::api())
            .arg(common::directory())
            .arg(common::base64())
            .args(common::display_args())
            .next_display_order(0)
            .next_help_heading("RESTRICTION DETAILS")
            .args(common::restriction_details())
            .after_help(REGION_SPEC)
            .mut_arg("no-header", |a| a.hide(true))
    }
}

impl CliCommand for SeaplaneRestrictSet {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let mut req = RestrictReq::new(ctx)?;
        let restrict_ctx = ctx.restrict_ctx.get_mut_or_init();
        let api = restrict_ctx.api.as_ref().unwrap();
        let mut dir = restrict_ctx.directory.as_ref().unwrap().to_string();
        let details = restrict_ctx.restriction_details()?;

        req.set_api(api)?;
        req.set_directory(&dir)?;
        req.set_restriction(details)?;

        if ctx.args.out_format == OutputFormat::Table {
            if restrict_ctx.decode {
                let es = EncodedString::new(dir);
                dir = String::from_utf8_lossy(&es.decoded()?).to_string()
            };
            cli_println!("Set a restriction on directory {dir} in {api} API");
        } else {
            cli_println!("{}", json!({"set_restriction": {"api": api, "directory": dir} }))
        }
        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.restrict_ctx
            .init(RestrictCtx::from_restrict_set(&SeaplaneRestrictSetArgMatches(matches))?);
        ctx.args.out_format = matches.get_one("format").copied().unwrap_or_default();
        let mut restrict_ctx = ctx.restrict_ctx.get_mut_or_init();
        restrict_ctx.decode = matches.get_flag("decode");
        Ok(())
    }
}
