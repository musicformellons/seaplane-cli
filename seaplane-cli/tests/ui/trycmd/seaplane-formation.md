Be default, `seaplane formation` will display the help text:

```console
$ seaplane formation
? 2
Operate on local Formations Plans and remote Formation Instances of those Plans

Usage: seaplane[EXE] formation [OPTIONS] <COMMAND>

Commands:
  plan          Create a Seaplane Formation [aliases: create, add]
  delete        Deletes local Formation Plans and/or remote Formation Instances [aliases: del, remove, rm]
  fetch-remote  Fetch remote Formation Instances and create/synchronize local Plan definitions [aliases: fetch, sync, synchronize]
  land          Land a remote Formation Instance [aliases: stop]
  launch        Start a local Formation Plan creating a remote Formation Instance [aliases: start]
  list          List all local Formation Plans [aliases: ls]
  status        Show the status of a remote Formation Instance
  help          Print this message or the help of the given subcommand(s)

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
