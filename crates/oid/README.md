# Container Image Reference

![Rust Version][rustc-image]
[![crates.io][crate-image]][crate-link]
[![Dependency Status][deps-image]][deps-link]

A library for using and handling Seaplane Object IDs.

<!-- vim-markdown-toc GFM -->

* [About](#about)
* [The Pitch](#the-pitch)
* [The Anti-Pitch](#the-anti-pitch)
* [Example](#example)
* [License](#license)

<!-- vim-markdown-toc -->

## About

An Object ID (OID) is a [RFC4648] base32 (no padding) extended hex-encoded UUID
with a prefix separated by a `-`.

For example `tst-0ous781p4lu7v000pa2a2bn1gc`, by convention the prefix is three
ASCII lowercase characters, however that is a hard constraint of OIDs in
general. The current implementation limits prefixes to 3 characters, but prefix
limit could be exposed as a tunable if that need arises.

## The Pitch

OIDs allow a "human readable subject line" in the form of the prefix, where
actual data is a UUID. This means while debugging or
reviewing a system it's trivial to determine if an incorrect OID was passed in
a particular location by looking at the prefix. This isn't easily achievable
with bare UUIDs.

Base32 encoding the UUID also allows compressing the data into a smaller and
more familiar format for humans, akin to a commit hash.

## The Anti-Pitch

The downside to OIDs is a layer of indirection when handling IDs and values,
it's not immediately obvious that the OIDs are a prefixed UUID. Additionally,
the prefixes must be controlled in some manner including migrated on changes
which adds a layer of complexity at the application layer.

There is also additional processing overhead compared to a bare UUID in order
to encode/decode as well as handling the appending and removing the prefixes.

However, we believe the drawbacks to pale in comparison to the benefits derived
from the format.

## Example

```rust
use seaplane_oid::{error::*, Oid};

fn main() -> Result<()> {
    // OIDs can be created with a given prefix alone, which generates a new
    // UUID
    let oid = Oid::new("exm")?;
    println!("{oid}");

    // OIDs can be parsed from strings, however the "value" must be a valid
    // base32 encoded UUID
    let oid: Oid = "tst-0ous781p4lu7v000pa2a2bn1gc".parse()?;
    println!("{oid}");

    // OIDs can also be created from the raw parts
    let oid = Oid::with_uuid(
        "exm",
        "063dc3a0-3925-7c7f-8000-ca84a12ee183"
            .parse::<Uuid>()
            .unwrap(),
    )?;

    // One can retrieve the various parts of the OID if needed
    println!("Prefix: {}", oid.prefix());
    println!("Value: {}", oid.value());
    println!("UUID: {}", oid.uuid());

    Ok(())
}
```

## License

Licensed under the Apache License, Version 2.0, [LICENSE]. Copyright 2023 Seaplane IO, Inc.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/seaplane-oid.svg
[crate-link]: https://crates.io/crates/seaplane-oid
[deps-image]: https://deps.rs/repo/github/seaplane-io/seaplane/status.svg
[deps-link]: https://deps.rs/crate/seaplane-oid
[rustc-image]: https://img.shields.io/badge/rustc-1.60+-blue.svg

[//]: # (Links)

[LICENSE]: https://github.com/seaplane-io/seaplane/blob/main/LICENSE
[RFC4648]: https://datatracker.ietf.org/doc/html/rfc4648.html#section-7
