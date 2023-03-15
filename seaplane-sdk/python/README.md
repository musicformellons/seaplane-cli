# Seaplane Python SDK
[![PyPI](https://badge.fury.io/py/seaplane.svg)](https://badge.fury.io/py/seaplane)
[![Python](https://img.shields.io/pypi/pyversions/seaplane.svg?style=plastic)](https://badge.fury.io/py/seaplane)

Simple Python library to manage your resources at seaplane.

## What is Seaplane?

Seaplane is the global platform for building and scaling your application stack
without the complexity of managing cloud infrastructure.

It serves as a reference application for how our APIs can be utilized.

Not sure where to go to quickly run a workload on Seaplane? See our [Getting
Started] guide.

To build and test this software yourself, see the CONTRIBUTING document that is a peer to this one.

## Installation

```shell
pip install seaplane
```

## Configure your API KEY

* Set `SEAPLANE_API_KEY` environment variable.
* Use `config` object in order to set the api key.

```python
from seaplane import sea

sea.config.set_api_key("your_api_key")
```

## License

Licensed under the Apache License, Version 2.0, [LICENSE]. Copyright 2022 Seaplane IO, Inc.

[//]: # (Links)

[Seaplane]: https://seaplane.io/
[CLI]: https://github.com/seaplane-io/seaplane/tree/main/seaplane-cli
[SDK]: https://github.com/seaplane-io/seaplane/tree/main/seaplane
[Getting Started]: https://github.com/seaplane-io/seaplane/blob/main/seaplane-sdk/python/docs/quickstart.md
[CONTRIBUTING]: https://github.com/seaplane-io/seaplane/tree/main/seaplane-sdk/python/CONTRIBUTIONS.md
[LICENSE]: https://github.com/seaplane-io/seaplane/blob/main/LICENSE
