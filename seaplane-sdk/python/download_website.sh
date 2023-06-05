#!/bin/bash

URL="https://github.com/seaplane-io/seaplane/releases/download/sdk-py-v0.2.7/smartpipes-0.2.9.tar.gz"
curl -LO "$URL"

mkdir website
tar -xzf smartpipes-0.2.9.tar.gz -C website --strip-components=1

cd website || exit
npm install
npm run dev
