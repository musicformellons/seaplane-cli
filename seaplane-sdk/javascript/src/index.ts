import { log } from './logging';

if (!global.fetch) {
  log.warn("Fetch isn't supported in this Node version, using node-fetch-polyfill");

  const fetch = require('node-fetch-polyfill');
  global.fetch = fetch;
}

import Configuration from './configuration';
import Metadata from './api/metadata';
import Identify from './api/identify';
import Locks from './api/locks';
import Restrictions from './api/restrictions';

export { Configuration, Metadata, Identify, Locks, Restrictions };
