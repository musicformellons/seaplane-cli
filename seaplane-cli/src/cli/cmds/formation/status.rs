use clap::{ArgMatches, Command};

use crate::{
    cli::{cmds::formation::common, common as cli_common, CliCommand},
    error::Result,
    ops::formation::FormationNameId,
    printer::Output,
    Ctx, OutputFormat,
};

static LONG_ABOUT: &str = "Show the status of a remote Formation Instance

This command will display the status of one or more Formation Instances such as how many actual
containers are running compared to the minimum and maximums per Flight that the configuration
defines.";

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationStatus;

impl SeaplaneFormationStatus {
    pub fn command() -> Command {
        Command::new("status")
            .long_about(LONG_ABOUT)
            .about("Show the status of a remote Formation Instance")
            .arg(cli_common::format())
            .arg(
                common::name_id(false)
                    .help("The name or ID of the Formation to check, must be unambiguous"),
            )
            .arg(arg!(--("no-fetch")).help("Skip fetching and synchronizing of remote instances"))
    }
}

impl CliCommand for SeaplaneFormationStatus {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        common::run_fetch(ctx)?;

        let statuses = ctx.db.formations.statuses();

        match ctx.args.out_format {
            OutputFormat::Json => statuses.print_json(ctx)?,
            OutputFormat::Table => statuses.print_table(ctx)?,
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.out_format = matches.get_one("format").copied().unwrap_or_default();
        ctx.args.fetch = !matches.get_flag("no-fetch");
        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.name_id = matches.get_one::<FormationNameId>("name_id").cloned();
        Ok(())
    }
}
