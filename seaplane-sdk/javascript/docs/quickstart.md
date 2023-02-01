Quick Start
================

## Introduction

Get started quickly using the Seaplane SDK for Javascript/Typescript. This SDK makes it easy to integrate your Javascript/Typescript application, library, or script with Seaplane features and services.

This guide details the steps needed to install, update, and use the Seaplane SDK for Javascript/Typescript.

## Installation

### Install or Update Node

Before you install Seaplane SDK, install node v18 or later.

### Install Seaplane NPM package

```shell
npm install seaplane
```

### Configuration

Before using the Seaplane SDK, you need to set up authentication credentials for your Seaplane account using the Flightdeck WebUI.

You can retrieve the API Key from Flightdeck WebUI and pass it to the SDK, using the `seaplane.Configuration` class.

This is needed to set the API key before you start using the Seaplane Javascript/Typescript SDK services and features.

## Usage

To use the Seaplane Javascript/Typescript SDK, you must first import it.

Javascript: 
```javascript
const sea = require('seaplane')
```
Typescript:
```typescript
import sea from 'seaplane'
```

Configure the SDK to use your `API_KEY` which you can get from flightdeck WebUI.

```typescript
import sea from 'seaplane'

const config = new sea.Configuration({ 
  apiKey: "API_KEY"  
})
```

You are ready to use the Seaplane services like, metadata data store:

```typescript
import sea from 'seaplane'

const config = new sea.Configuration({ 
  apiKey: "API_KEY"  
})
const metadata = new sea.Metadata(config)

metadata.set({ key: "key", value: "value" })
metadata.get({ key: "key" })
metadata..get_page()
```

And that's it! You've got your Seaplane Javascript/Typescript SDK ready to go.