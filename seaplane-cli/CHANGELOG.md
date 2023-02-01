# Seaplane CLI Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.5.0 - 01 Feb 2023

### Features

* Automatically detect, and upgrade local state ([#306](https://github.com/seaplane-io/seaplane/pull/306))

### Improvements

* Combine local state (`flights.json` and `formations.json` into a single
  `state.json`) and version the state schema
  ([#305](https://github.com/seaplane-io/seaplane/pull/305))

### Maintenance

* Fix lints, upgrade `toml` crate and fix debug output ([#304](https://github.com/seaplane-io/seaplane/pull/304))

## 0.4.1 - 18 Jan 2023

### Bug Fixes

* Updated to Seaplane SDK 0.4.0 to handle Locks API naming change

## 0.4.0 - 13 Jan 2023

### Features

* Add Javascript SDK ([#268](https://github.com/seaplane-io/seaplane/pull/268))
* Adds Locks Javascript SDK ([#272](https://github.com/seaplane-io/seaplane/pull/272))

### Fixes and Improvements

* Fix arg parsing testing ([#282](https://github.com/seaplane-io/seaplane/pull/282))
* Fix ES module using fetch instead of Axios SDK ([#283](https://github.com/seaplane-io/seaplane/pull/283))
* Fixes unbound variable in Justfile ([#273](https://github.com/seaplane-io/seaplane/pull/273))
* Fix `just` workflow on Windows ([#270](https://github.com/seaplane-io/seaplane/pull/270))
* removes `dist/` that was mistakenly added ([#269](https://github.com/seaplane-io/seaplane/pull/269))

### Maintenance

* Minor CI improvements ([#285](https://github.com/seaplane-io/seaplane/pull/285))
* Factor out `image-ref` module into crate [`container-image-ref`](https://crates.io/crates/container-image-ref)([#276](https://github.com/seaplane-io/seaplane/pull/276))
* Bump pinned `rustc` version ([#280](https://github.com/seaplane-io/seaplane/pull/280))
* Bump Rust Dependencies ([#275](https://github.com/seaplane-io/seaplane/pull/275))
* Use BuildJet runners in CI ([#277](https://github.com/seaplane-io/seaplane/pull/277))
* Add Python and JS to Just ([#274](https://github.com/seaplane-io/seaplane/pull/274))
* Use `Compress-Archive` on Windows instead of `zip` in CI ([#271](https://github.com/seaplane-io/seaplane/pull/271))
* Fix python SDK PR workflow and workflow typos ([#265](https://github.com/seaplane-io/seaplane/pull/265))
* Only include tagged CLI artifacts in release ([#264](https://github.com/seaplane-io/seaplane/pull/264))

### Documentation

* Removing link from Seaplane ([#267](https://github.com/seaplane-io/seaplane/pull/267))
* Fixes extraction directory on Linux ([#266](https://github.com/seaplane-io/seaplane/pull/266))

## 0.3.1 - 03 Nov 2022

### Bug Fixes

- Fixes panic when using legacy `--minimum` and `--maximum` when defining Flight Plans ([#263](https://github.com/seaplane-io/seaplane/pull/263))

## 0.3.0 - 01 Nov 2022

### Breaking Changes

- Default container image registry has changed from `registry.hub.docker.com/` to `registry.cplane.cloud/` ([#255](https://github.com/seaplane-io/seaplane/pull/255))

### Features

- *(Configuration)* Default container image registry can be set ([#254](https://github.com/seaplane-io/seaplane/pull/254))

## 0.2.0 - 21 Oct 2022

- Initial Public Release
