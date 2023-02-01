use std::io::{self, BufRead};

use clap::{ArgMatches, Command};
use seaplane::api::identity::v0::TokenRequest;

use crate::{
    cli::CliCommand,
    config::RawConfig,
    context::Ctx,
    error::{CliError, CliErrorKind, Context, Result},
    fs::{FromDisk, ToDisk},
    printer::{Color, OutputFormat},
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneAccount;

impl SeaplaneAccount {
    pub fn command() -> Command {
        Command::new("account")
            .visible_alias("acct")
            .about("Operate on Seaplane account details, including access tokens")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(SeaplaneAccountLogin::command())
            .subcommand(SeaplaneAccountToken::command())
    }
}

impl CliCommand for SeaplaneAccount {
    fn next_subcmd<'a>(
        &self,
        matches: &'a ArgMatches,
    ) -> Option<(Box<dyn CliCommand>, &'a ArgMatches)> {
        match matches.subcommand() {
            Some(("login", m)) => Some((Box::new(SeaplaneAccountLogin), m)),
            Some(("token", m)) => Some((Box::new(SeaplaneAccountToken), m)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneAccountToken;

impl SeaplaneAccountToken {
    pub fn command() -> Command {
        Command::new("token").arg(arg!(--json - ('j')).help(
            "Returns the access token in a JSON object also containing tenant ID and subdomain",
        ))
    }
}

impl CliCommand for SeaplaneAccountToken {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let mut builder = TokenRequest::builder().api_key(ctx.args.api_key()?);

        if let Some(url) = ctx.identity_url.as_ref() {
            builder = builder.base_url(url);
        }

        #[cfg(feature = "allow_insecure_urls")]
        {
            builder = builder.allow_http(ctx.insecure_urls);
        }
        #[cfg(feature = "allow_invalid_certs")]
        {
            builder = builder.allow_invalid_certs(ctx.invalid_certs);
        }

        let t = builder.build().map_err(CliError::from)?;

        if ctx.args.out_format == OutputFormat::Json {
            cli_println!("{}", serde_json::to_string(&t.access_token_json()?)?);
        } else {
            cli_println!("{}", t.access_token()?);
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        if matches.get_flag("json") {
            ctx.args.out_format = OutputFormat::Json;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneAccountLogin;

impl SeaplaneAccountLogin {
    pub fn command() -> Command {
        Command::new("login").arg(arg!(--force - ('f')).help("Override any existing API key"))
    }
}

impl CliCommand for SeaplaneAccountLogin {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        if ctx.args.stateless {
            cli_bail!("'--stateless' cannot be used with 'seaplane account login'");
        }
        let mut cfg = if let Some(f) = ctx.conf_files().first() {
            RawConfig::load(f)?
        } else {
            // Try and load whatever the defaults are. NOTE this does not update the
            // `ctx.conf_dirs`. However this is fine because the remaining code paths after this
            // don't try and access them.
            RawConfig::load_all()?
        };

        if let Some(key) = cfg.account.api_key {
            if ctx.args.force {
                cli_warn!(@prefix, "overwriting API key ");
                cli_warn!(@Green, "{key} ");
                cli_warn!("due to ");
                cli_warnln!(@noprefix, @Green, "--force");
            } else {
                return Err(CliErrorKind::ExistingValue("an API key")
                    .into_err()
                    .context("(hint: add '")
                    .color_context(Color::Green, "--force")
                    .context("' to overwrite it)\n"));
            }
        }
        cli_println!("Enter an API key below.");
        cli_print!("(hint: it can be found by visiting ");
        cli_print!(@Green, "https://flightdeck.cplane.cloud/");
        cli_println!(")\n");

        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();
        if let Some(line) = lines.next() {
            ctx.args.api_key = Some(line?);
        }

        cfg.account.api_key = ctx.args.api_key.clone();

        cfg.persist()?;

        cli_println!("Successfully saved the API key!");

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.force = matches.get_flag("force");
        Ok(())
    }
}
