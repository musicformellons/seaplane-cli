use clap::{ArgMatches, Command};
use seaplane::{
    api::{
        compute::v1::{ActiveConfiguration, ActiveConfigurations},
        ApiErrorKind,
    },
    error::SeaplaneError,
};

use crate::{
    api::FormationsReq,
    cli::{
        cmds::formation::SeaplaneFormationFetch,
        errors,
        validator::{validate_formation_name, validate_name_id},
        CliCommand,
    },
    context::Ctx,
    error::{CliErrorKind, Context, Result},
    printer::{Color, Pb},
};

static LONG_ABOUT: &str = "Start a local Formation Plan creating a remote Formation Instance

In many cases, or at least initially a local Formation Plan may only have a single
Formation Configuration. In these cases this command will set that one configuration to active
creating a remote Formation Instance with a single configuration.

Things become slightly more complex when there are multiple Formation Configurations. Let's
look at each possibility in turn.

\"Local Only\" Configs Exist:

A \"Local Only\" Config is a configuration that exists in the local database, but has not (yet)
been uploaded to the Seaplane Cloud.

In these cases the configurations will be sent to the Seaplane Cloud, and set to active. If the
Seaplane Cloud already has configurations for the given Formation (either active or inactive),
these new configurations will be appended, and traffic will be balanced between any *all*
configurations.

\"Remote Active\" Configs Exist:

A \"Remote Active\" Config is a configuration that the Seaplane Cloud is aware of, and actively
sending traffic to.

These configurations will remain active and traffic will be balanced between any *all*
configurations.

\"Remote Inactive\" Configs Exist:

A \"Remote Inactive\" Config is a configuration that the Seaplane Cloud is aware of, and but not
sending traffic to.

These configurations will be made active. If the Seaplane Cloud already has active
configurations for the given Formation, these newly activated configurations will be appended,
and traffic will be balanced between any *all* configurations. ";

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationLaunch;

impl SeaplaneFormationLaunch {
    pub fn command() -> Command {
        let validator = |s: &str| validate_name_id(validate_formation_name, s);

        // TODO: make it possible to selectively start only *some* configs
        Command::new("launch")
            .visible_alias("start")
            .about("Start a local Formation Plan creating a remote Formation Instance")
            .long_about(LONG_ABOUT)
            .arg(
                arg!(formation =["NAME|ID"] required)
                    .value_parser(validator)
                    .help("The name or ID of the Formation Plan to launch and create an Instance of"),
            )
            .arg(
                arg!(--all - ('a'))
                    .help("Launch all matching local Formation Plans even when the name or ID is ambiguous"),
            )
            .arg(arg!(--fetch|sync|synchronize - ('F')).help(
                "Fetch remote Formation Instances and synchronize local Plan definitions prior to attempting to launch",
            ))
            .arg(
                arg!(--grounded).help(
                    "Upload the configuration(s) defined in this local Formation Plan to Seaplane but *DO NOT* set them to active",
                ),
            )
    }
}

impl CliCommand for SeaplaneFormationLaunch {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let name = ctx.args.name_id.as_ref().unwrap().clone();
        if ctx.args.fetch {
            let old_name = ctx.args.name_id.take();
            let old_ir = ctx.internal_run;
            ctx.internal_run = true;
            SeaplaneFormationFetch.run(ctx)?;
            ctx.internal_run = old_ir;
            ctx.args.name_id = old_name;
        }
        // Get the indices of any formations that match the given name/ID
        let indices = if ctx.args.all {
            ctx.db.formations.formation_indices_of_left_matches(&name)
        } else {
            ctx.db.formations.formation_indices_of_matches(&name)
        };

        match indices.len() {
            0 => errors::no_matching_item(name, false, ctx.args.all)?,
            1 => (),
            _ => {
                // TODO: and --force
                if !ctx.args.all {
                    errors::ambiguous_item(name, true)?;
                }
            }
        }

        let pb = Pb::new(ctx);
        let grounded = ctx.formation_ctx.get_or_init().grounded;
        let mut req = FormationsReq::new_delay_token(ctx)?;
        for idx in indices {
            // re unwrap: the indices returned came from Formations so they have to be valid
            let formation = ctx.db.formations.get_formation(idx).unwrap();
            let formation_name = formation.name.as_ref().unwrap().clone();
            req.set_name(&formation_name)?;

            // Get the local configs that don't exist remote yet
            let cfgs_ids = if grounded {
                formation.local_only_configs()
            } else {
                formation.local_or_grounded_configs()
            };

            if cfgs_ids.is_empty() {
                if ctx.args.fetch && grounded {
                    cli_println!("Formation Plan '{formation_name}' is already uploaded and in status Grounded");
                    return Ok(());
                } else {
                    return Err(CliErrorKind::OneOff(
                        "Cannot find local Formation Plan '{formation_name}'".into(),
                    )
                    .into_err())
                    .context(
                        "(hint: you can try adding '--fetch' to synchronize remote instances)\n",
                    );
                }
            }

            // Keep track if we had to make a brand new formation or not
            let mut created_new = false;
            let mut has_public_endpoints = false;
            let mut cfg_uuids = Vec::new();

            // Add those configurations to this formation
            'inner: for id in &cfgs_ids {
                if let Some(cfg) = ctx.db.formations.get_configuration(id) {
                    has_public_endpoints = cfg.model.public_endpoints().count() > 0;
                    if let Some(flight) = cfg.model.public_endpoints().find_map(|(_, dst)| {
                        if !ctx.db.formations.has_flight(&dst.flight_name) {
                            Some(dst.flight_name.clone())
                        } else {
                            None
                        }
                    }) {
                        return Err(CliErrorKind::EndpointInvalidFlight(flight).into_err()
                            .context("perhaps the Flight Plan exists, but only in a remote Formation Instance?\n")
                            .context("(hint: use '")
                            .color_context(Color::Yellow, "--fetch")
                            .context("' to synchronize local Plan definitions with remote Instances)\n")
                            .context("(hint: alternatively, you create the Flight Plan with '")
                            .color_context(Color::Green, "seaplane flight plan")
                            .context("')\n"));
                    }

                    // We don't set the configuration to active because we'll be doing that to
                    // *all* formation configs in a minute
                    pb.set_message("Searching for existing Formations...");
                    if let Err(e) = req.add_configuration(&cfg.model, false) {
                        match e.kind() {
                            CliErrorKind::Seaplane(SeaplaneError::ApiResponse(ae))
                                if ae.kind == ApiErrorKind::NotFound =>
                            {
                                // If the formation didn't exist, create it
                                pb.set_message("Creating new Formation Instance...");
                                let cfg_uuid = req.create(&cfg.model, !grounded)?;
                                cfg_uuids.extend(cfg_uuid);
                                ctx.db.formations.add_in_air_by_name(&formation_name, *id);
                                created_new = true;
                                break 'inner;
                            }
                            _ => return Err(e),
                        }
                    } else {
                        pb.set_message("Found existing Formation Instance...");
                        ctx.db.formations.add_grounded_by_name(&formation_name, *id);
                    }
                } else {
                    // TODO: Inform the user of possible error? Somehow there is no config by the
                    // ID? This is an internal error that shoulnd't happen. We got the ID from our
                    // own internal state, if that's wrong we have big issues
                    unreachable!()
                }
            }

            // If the user passed `--grounded` they don't want the configuration to be set to
            // active
            if !grounded && !created_new {
                // Get all configurations for this Formation
                pb.set_message("Synchronizing Formation Configurations...");
                cfg_uuids.extend(
                    req.list_configuration_ids()
                        .context("Context: failed to retrieve Formation Configuration IDs\n")?,
                );
                let mut active_configs = ActiveConfigurations::new();
                for uuid in &cfg_uuids {
                    #[cfg_attr(not(feature = "unstable"), allow(unused_mut))]
                    let mut cfg = ActiveConfiguration::builder().uuid(*uuid);
                    #[cfg(feature = "unstable")]
                    {
                        cfg = cfg.traffic_weight(1.0);
                    }
                    active_configs.add_configuration_mut(cfg.build()?);
                }
                pb.set_message("Adding Formation Configurations to remote Instance...");
                req.set_active_configurations(&active_configs, false)
                    .context("Context: failed to start Formation\n")?;
                for id in cfgs_ids {
                    ctx.db.formations.add_in_air_by_name(&formation_name, id);
                }
            }
            pb.set_message("Getting Formation URL...");
            let domain = req.get_metadata()?.url;

            pb.finish_and_clear();
            cli_print!("Successfully Launched remote Formation Instance '");
            cli_print!(@Green, "{}", &ctx.args.name_id.as_ref().unwrap());
            if cfg_uuids.is_empty() {
                cli_println!("'");
            } else {
                cli_println!("' with Configuration UUIDs:");
                for uuid in cfg_uuids {
                    cli_println!(@Green, "{uuid}");
                }
            }

            cli_print!("The remote Formation Instance URL is ");
            cli_println!(@Green, "{domain}");
            cli_println!(
                "(hint: it may take up to a minute for the Formation to become fully online)"
            );
            if !has_public_endpoints {
                cli_println!("(hint: there are no public endpoints configured, the Formation will not be reachable from the public internet)");
            }
            cli_print!("(hint: check the status of this Formation Instance with '");
            cli_print!(@Green, "seaplane formation status {formation_name}");
            cli_println!("')");
        }

        ctx.persist_state()?;

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.name_id = matches
            .get_one::<String>("formation")
            .map(ToOwned::to_owned);
        ctx.args.fetch = matches.get_flag("fetch");
        let fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.grounded = matches.get_flag("grounded");
        Ok(())
    }
}
