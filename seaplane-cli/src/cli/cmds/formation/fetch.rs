use std::collections::HashSet;

use clap::{ArgMatches, Command};

use crate::{api::FormationsReq, cli::CliCommand, error::Result, printer::Pb, Ctx};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationFetch;

impl SeaplaneFormationFetch {
    pub fn command() -> Command {
        // TODO: add a --no-overwrite or similar
        Command::new("fetch-remote")
            .visible_aliases(["fetch", "sync", "synchronize"])
            .about("Fetch remote Formation Instances and create/synchronize local Plan definitions")
            .override_usage("
    seaplane formation fetch-remote
    seaplane formation fetch-remote [NAME|ID]",
            )
            .arg(
                arg!(formation = ["NAME|ID"])
                    .help("The NAME or ID of the remote Formation Instance to fetch, omit to fetch all Formation Instances"),
            )
    }
}

impl CliCommand for SeaplaneFormationFetch {
    // TODO: async
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let pb = Pb::new(ctx);
        pb.set_message("Gathering Formation Names...");

        let mut req = FormationsReq::new_delay_token(ctx)?;
        if let Some(name) = &ctx.args.name_id {
            req.set_name(name)?;
        }
        let names = req.get_formation_names()?;

        pb.set_message("Syncing Formations...");
        // This gets us everything the API knows about, but with totally new local IDs. So we need
        // to ignore those except how they relate to eachother (i.e. they won't match anything in
        // our local DB, but they will match within these instances returned by the API).
        //
        // We need to map them to our OWN local IDs and update the DB.
        let mut remote_instances = req.get_all_formations(&names, &pb)?;

        // Keep track of what new items we've downloaded that our local DB didn't know about
        let mut flights_added = HashSet::new();
        let mut formations_added = HashSet::new();
        let mut configs_updated = HashSet::new();

        // Start going through the instances by formation
        for formation in remote_instances.formations.iter_mut() {
            // Loop through all the Formation Configurations defined in this Formation
            for cfg in formation.configs().iter().filter_map(|id| {
                // get the index of the Config where the ID matches
                if let Some(i) = remote_instances
                    .configurations
                    .iter()
                    .enumerate()
                    .find_map(|(i, cfg)| if &cfg.id == id { Some(i) } else { None })
                {
                    // Map a config ID to an actual Config. We have to use these long chained calls
                    // so Rust can tell that `formations` itself isn't being
                    // borrowed, just it's fields.
                    Some(remote_instances.configurations.swap_remove(i))
                } else {
                    None
                }
            }) {
                // Add or update all flights this configuration references
                for flight in cfg.model.flights() {
                    // If the name AND image match something in our local DB we update, otherwise
                    // we assume it's new and add it to our local DB, new ID and all
                    let names_ids = ctx.db.flights.update_or_create_flight(flight);
                    flights_added.extend(names_ids);
                }

                // Keep track of the old ID incase we need to replace it
                let old_id = cfg.id;
                // if we only updated, and didn't create, we need to replace the random local ID
                // that was assigned when we downloaded all the configs, with the *real* local ID
                if let Some(real_id) = ctx.db.formations.update_or_create_configuration(cfg) {
                    formation.replace_id(&old_id, real_id);
                    configs_updated.insert((formation.name.clone().unwrap(), real_id));
                }
            }
            // Add or update the formation itself (which is really just a list of configuration
            // local IDs)
            if let Some(id) = ctx
                .db
                .formations
                .update_or_create_formation(formation.clone())
            {
                formations_added.insert((formation.name.clone().unwrap(), id));
            }
        }

        pb.finish_and_clear();

        if !ctx.internal_run {
            let mut count = 0;
            for (name, id) in formations_added {
                count += 1;
                cli_print!("Successfully synchronized Formation Instance '");
                cli_print!(@Green, "{name}");
                cli_print!("' with local Formation ID '");
                cli_print!(@Green, "{}", &id.to_string()[..8]);
                cli_println!("'");
            }
            for (name, id) in configs_updated {
                count += 1;
                cli_print!("Successfully synchronized Formation Configuration in Formation '");
                cli_print!(@Green, "{name}");
                cli_print!("' with local Formation Configuration ID '");
                cli_print!(@Green, "{}", &id.to_string()[..8]);
                cli_println!("'");
            }
            for (name, id) in flights_added {
                count += 1;
                cli_print!("Successfully synchronized Flight Plan '");
                cli_print!(@Green, "{name}");
                cli_print!("' with local Flight Plan ID '");
                cli_print!(@Green, "{}", &id.to_string()[..8]);
                cli_println!("'!");
            }
            if names.is_empty() {
                cli_println!("No remote Formation Instances found");
            } else if count > 0 {
                cli_println!("\nSuccessfully fetched {count} items");
            } else {
                cli_println!("All local definitions are up to date!");
            }
        }

        ctx.persist_state()?;

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.name_id = matches
            .get_one::<String>("formation")
            .map(ToOwned::to_owned);
        Ok(())
    }
}
