Without any arguments

```console
$ seaplane
? 2
Seaplane CLI for managing resources on the Seaplane Cloud

Usage: seaplane[EXE] [OPTIONS] <COMMAND>

Commands:
  account           Operate on Seaplane account details, including access tokens [aliases: acct]
  formation         Operate on local Formations Plans and remote Formation Instances of those Plans
  init              Create the Seaplane directory structure at the appropriate locations
  license           Print license information
  metadata          Operate on metadata key-value pairs using the Global Data Coordination API [aliases: meta, md]
  locks             Operate on the Locks API
  restrict          Restrict the placement of data for Global Data Coordination API
  shell-completion  Generate shell completion scripts for the Seaplane CLI
  help              Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...        Display more verbose output
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
      --no-color          Do not color output (alias for --color=never)
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -S, --stateless         Ignore local state files, do not read from or write to them
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

```
