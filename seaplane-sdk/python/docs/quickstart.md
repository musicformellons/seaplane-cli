Quick Start
================

## Introduction

Get started quickly using the Seaplane SDK for Python. This SDK makes it easy to integrate your Python application, library, or script with Seaplane features and services.

This guide details the steps needed to install, update, and use the Seaplane SDK for Python.

## Installation

### Install or Update Python

Before you install Seaplane Python SDK, install Python 3.7 or later.

### Install Seaplane Python SDK

```shell
pip install seaplane
```

### Configuration

Before using the Python SDK, you need to set up authentication credentials for your Seaplane account using the Flightdeck WebUI.

You can retrieve the API Key from Flightdeck WebUI and pass it to the Python SDK, using the `Configuration.set_api_key` method.

This is needed to set the API key before you start using the Seaplane Python SDK services and features.

## Usage

To use the Seaplane Python SDK, you must first import it.

```python
from seaplane import sea
```

Configure the SDK to use your API_KEY which you can get from flightdeck WebUI.

```python
from seaplane import sea

sea.config.set_api_key("your_api_key")
```

You are ready to use the Seaplane services like, metadata data store:

```python
from seaplane import sea
from seaplane.model import Key, KeyValue

sea.metadata.set(KeyValue(b"key", b"value"))
sea.metadata.get(Key(b"key"))
sea.metadata.get_page()
```

And that's it! You've got your Seaplane Python SDK ready to go.