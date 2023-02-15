```console
$ seaplane formation launch -h
Start a local Formation Plan creating a remote Formation Instance

Usage: seaplane[EXE] formation launch [OPTIONS] <NAME>

Arguments:
  <NAME>  The name of the Formation Plan to launch and create an Instance of

Options:
  -a, --all               Operate on all matching local Formation Plans even when the name or ID is ambiguous
  -v, --verbose...        Display more verbose output
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
      --no-color          Do not color output (alias for --color=never)
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -S, --stateless         Ignore local state files, do not read from or write to them
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

```

```console
$ seaplane formation launch --help
Start a local Formation Plan creating a remote Formation Instance

Usage: seaplane[EXE] formation launch [OPTIONS] <NAME>

Arguments:
  <NAME>
          The name of the Formation Plan to launch and create an Instance of

Options:
  -a, --all
          Operate on all matching local Formation Plans even when the name or ID is ambiguous

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

  -q, --quiet...
          Suppress output at a specific level and below
          
          More uses suppresses higher levels of output
              -q:   Only display WARN messages and above
              -qq:  Only display ERROR messages
              -qqq: Suppress all output

      --color <COLOR>
          Should the output include color?
          
          [default: auto]
          [possible values: always, ansi, auto, never]

      --no-color
          Do not color output (alias for --color=never)

  -A, --api-key <STRING>
          The API key associated with a Seaplane account used to access Seaplane API endpoints
          
          The value provided here will override any provided in any configuration files.
          A CLI provided value also overrides any environment variables.
          One can use a special value of '-' to signal the value should be read from STDIN.
          
          [env: SEAPLANE_API_KEY]

  -S, --stateless
          Ignore local state files, do not read from or write to them

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```
