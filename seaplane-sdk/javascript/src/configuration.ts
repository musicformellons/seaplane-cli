const SEAPLANE_COMPUTE_API_ENDPOINT = 'https://compute.cplane.cloud/v1';
const SEAPLANE_COORDINATION_API_ENDPOINT = 'https://metadata.cplane.cloud/v1';
const SEAPLANE_IDENTIFY_API_ENDPOINT = 'https://flightdeck.cplane.cloud/v1';

type ConfigValues = {
  apiKey: string;
  identifyEndpoint?: string;
  computeEndpoint?: string;
  coordinationEndpoint?: string;
  autoRenew?: boolean;
  accessToken?: string;
};

import Identify from './api/identify';
import { log, SeaLogger } from './logging';

export default class Configuration {
  apiKey: string;
  identifyEndpoint: string;
  computeEndpoint: string;
  coordinationEndpoint: string;
  autoRenew: boolean;
  accessToken?: string;
  identify: Identify;

  constructor(configValues: ConfigValues) {
    this.apiKey = configValues.apiKey;
    this.identifyEndpoint = configValues.identifyEndpoint || SEAPLANE_IDENTIFY_API_ENDPOINT;
    this.computeEndpoint = configValues.computeEndpoint || SEAPLANE_COMPUTE_API_ENDPOINT;
    this.coordinationEndpoint = configValues.coordinationEndpoint || SEAPLANE_COORDINATION_API_ENDPOINT;
    this.autoRenew = configValues.autoRenew || true;
    this.accessToken = configValues.accessToken;
    this.identify = new Identify(this);
  }

  values(): ConfigValues {
    return {
      apiKey: this.apiKey,
      identifyEndpoint: this.identifyEndpoint,
      computeEndpoint: this.computeEndpoint,
      coordinationEndpoint: this.coordinationEndpoint,
      autoRenew: this.autoRenew,
    };
  }

  logLevel(level: string) {
    log.level(level);

    if (level == SeaLogger.DEBUG) {
      log.debug('Seaplane debug activated');
      log.debug(`Identify endpoint: ${this.identifyEndpoint}`);
      log.debug(`Compute endpoint: ${this.computeEndpoint}`);
      log.debug(`Coordination endpoint: ${this.coordinationEndpoint}`);
    }
  }

  logEnable(enable: boolean) {
    if (enable) {
      log.enable();
    } else {
      log.disable();
    }
  }
}
