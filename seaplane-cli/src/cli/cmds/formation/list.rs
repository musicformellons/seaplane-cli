use clap::{ArgMatches, Command};

use crate::{
    cli::{cmds::formation::common, common as cli_common, CliCommand},
    error::Result,
    printer::Output,
    Ctx, OutputFormat,
};

static LONG_ABOUT: &str = "List all local Formation Plans

This command will display the status each of your Formation Plans. The Formations displayed come
from the local database of known Formations. You may wish to update the local database with Remote
Formation Instances as well by either first running:

$ seaplane formation fetch-remote

OR including `--fetch` such as:

$ seaplane formation list --fetch

After which your local database of Formations and Flights will contain all remote Formation
Instances as well.";

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationList;

impl SeaplaneFormationList {
    pub fn command() -> Command {
        Command::new("list")
            .visible_alias("ls")
            .long_about(LONG_ABOUT)
            .about("List all local Formation Plans")
            .arg(common::fetch(true))
            .arg(cli_common::format())
    }
}

impl CliCommand for SeaplaneFormationList {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        if ctx.args.stateless && !ctx.args.fetch {
            cli_eprint!(@Red, "error: ");
            cli_eprint!("'");
            cli_eprint!(@Yellow, "seaplane formation list ");
            cli_eprint!(@Red, "--stateless");
            cli_eprint!("' is useless without '");
            cli_eprint!(@Green, "--fetch");
            cli_eprintln!("'");
            cli_eprintln!("(hint: 'seaplane formation list' only looks at local Plans)");
            cli_eprint!("(hint: 'seaplane formation list");
            cli_eprint!(@Green, "--fetch");
            cli_eprintln!("' also fetches remote Instances)");
            std::process::exit(1);
        }

        common::run_fetch(ctx)?;

        match ctx.args.out_format {
            OutputFormat::Json => ctx.db.formations.print_json(ctx)?,
            OutputFormat::Table => ctx.db.formations.print_table(ctx)?,
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.out_format = matches.get_one("format").copied().unwrap_or_default();
        ctx.args.fetch = matches.get_flag("fetch");
        Ok(())
    }
}
