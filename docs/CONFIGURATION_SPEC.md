# Seaplane Configuration Specification

This document describes the `seaplane` configuration file and it's specification.


<!-- vim-markdown-toc GFM -->

* [Search Locations](#search-locations)
    * [Linux](#linux)
    * [macOS](#macos)
    * [Windows](#windows)
* [Format](#format)
* [Sections and Field Reference](#sections-and-field-reference)
    * [The `[seaplane]` Section](#the-seaplane-section)
        * [The `color` Field](#the-color-field)
        * [The `default-registry` Field](#the-default-registry-field)
    * [The `[account]` Section](#the-account-section)
        * [The `api-key` Field](#the-api-key-field)
    * [The `[api]` Section](#the-api-section)
        * [The `compute-url` Field](#the-compute-url-field)
        * [The `metadata-url` Field](#the-metadata-url-field)
        * [The `identity-url` Field](#the-identity-url-field)
    * [The `[danger-zone]` Section](#the-danger-zone-section)
        * [The `allow-insecure-urls` Field](#the-allow-insecure-urls-field)
        * [The `allow-invalid-certs` Field](#the-allow-invalid-certs-field)

<!-- vim-markdown-toc -->

## Search Locations

On startup, `seaplane` will check in platform specific locations for a configuration file. If
found, the file will be loaded.

The following locations and order are used.

### Linux

- `$XDG_CONFIG_HOME/seaplane/`
- `$HOME/.config/seaplane/`
- `$HOME/.seaplane/`

### macOS

- `$HOME/Library/ApplicationSupport/io.Seaplane.seaplane/`
- `$HOME/.config/seaplane/`
- `$HOME/.seaplane/`

### Windows

- `%RoamingAppData%/Seaplane/seaplane/config/`
- `$HOME/.config/seaplane/`
- `$HOME/.seaplane/`

Alternatively, a custom file may be specified at the command line use the
appropriate flags.

## Format

The `seaplane` configuration file is in [TOML][toml] format and consists of the
following sections.

!!! Note
    Most sections may be omitted, and their default values will be used.

## Sections and Field Reference

The following sections enumerate the tables, their fields and values for all
configurable options.

### The `[seaplane]` Section

The first section in a `seaplane.toml` is the `[seaplane]` table which contains
various settings that apply to your local invocations of the `seaplane` CLI or
interactions with the APIs.

#### The `color` Field

Controls whether or not the output contains color. The field is a string and
may contain one of the following values:

- `always`: The output will be colored even when STDOUT or STDERR is not a TTY
- `auto` (default): The output will be colored only if STDOUT/STDERR is a TTY
  and the `NO_COLOR` environment variable is not set
- `ansi`: If output is selected, always use ANSI color codes even on Windows
  (instead of console API calls)
- `never`: Do not color output

#### The `default-registry` Field

Names which container image registry will be inferred if not provided for
commands and CLI flags that take a container image reference.

Default value: `registry.cplane.cloud`

For example, `seaplane formation plan -flight image=nginx:latest` will be
expanded to `registry.cplane.cloud/nginx:latest`

### The `[account]` Section

The second section in a `seaplane.toml` is the `[account]` table which contains
your account information for accessing Seaplane resources and APIs.

#### The `api-key` Field

Your API key is used to authenticate yourself to the Seaplane Authentication
APIs which in turn send back a short lived Authentication Token which
authorizes actions against resources you own such as your Formations and
Flights.

The `api-key` field is a string.

### The `[api]` Section

The third section in a `seaplane.toml` is the `[api]` table which contains the
base URL used to reach each of the resources APIs.

#### The `compute-url` Field

The base URL used for the Compute API.

Default: `https://compute.cplane.cloud/`

#### The `metadata-url` Field

The base URL used for the Metadata Service API.

Default: `https://metadata.cplane.cloud/`

#### The `identity-url` Field

The base URL used for the Identity API.

Default: `https://flightdeck.cplane.cloud/`

### The `[danger-zone]` Section

This section contains settings that are potentially dangerous, or not meant to
be used by the general public except in rare circumstances. Although sometimes
these options are useful, especially in cases like development.

#### The `allow-insecure-urls` Field

Controls whether API endpoints are allowed to be HTTP or are instead required
to be HTTPS only.

Default: `false`

> **Note**
> Only available on crate feature `allow_insecure_urls`

#### The `allow-invalid-certs` Field

Controls whether invalid TLS certificate are accepted.

Default: `false`

> **Note**
> Only available on crate feature `allow_invalid_certs`

[//]: # (links)

[toml]: https://toml.io/
