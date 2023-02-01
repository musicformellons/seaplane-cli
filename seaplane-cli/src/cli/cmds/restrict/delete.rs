use clap::{ArgMatches, Command};
use serde_json::json;

use crate::{
    api::RestrictReq,
    cli::cmds::restrict::{common, common::SeaplaneRestrictCommonArgMatches, CliCommand},
    context::{Ctx, RestrictCtx},
    error::Result,
    ops::EncodedString,
    printer::OutputFormat,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneRestrictDelete;

impl SeaplaneRestrictDelete {
    pub fn command() -> Command {
        Command::new("delete")
            .visible_aliases(["del", "remove", "rm"])
            .about("Delete a restriction on directory")
            .arg(common::api())
            .arg(common::directory())
            .arg(common::base64())
            .args(common::display_args())
            .mut_arg("no-header", |a| a.hide(true))
    }
}

impl CliCommand for SeaplaneRestrictDelete {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let mut req = RestrictReq::new(ctx)?;
        let restrict_ctx = ctx.restrict_ctx.get_mut_or_init();
        let api = restrict_ctx.api.as_ref().unwrap();
        let mut dir = restrict_ctx.directory.as_ref().unwrap().to_string();
        req.set_api(api)?;
        req.set_directory(&dir)?;
        req.delete_restriction()?;

        if ctx.args.out_format == OutputFormat::Table {
            if restrict_ctx.decode {
                let es = EncodedString::new(dir);
                dir = String::from_utf8_lossy(&es.decoded()?).to_string()
            };
            cli_println!("Deleted a restriction on directory {dir} in {api} API");
        } else {
            cli_println!("{}", json!({"deleted_restriction": {"api": api, "directory": dir} }))
        }
        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.restrict_ctx
            .init(RestrictCtx::from_restrict_common(&SeaplaneRestrictCommonArgMatches(matches))?);
        ctx.args.out_format = matches.get_one("format").copied().unwrap_or_default();
        let mut restrict_ctx = ctx.restrict_ctx.get_mut_or_init();
        restrict_ctx.decode = matches.get_flag("decode");
        Ok(())
    }
}
