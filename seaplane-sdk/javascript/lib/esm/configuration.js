const SEAPLANE_COMPUTE_API_ENDPOINT = "https://compute.cplane.cloud/v1";
const SEAPLANE_COORDINATION_API_ENDPOINT = "https://metadata.cplane.cloud/v1";
const SEAPLANE_IDENTIFY_API_ENDPOINT = "https://flightdeck.cplane.cloud";

export default class Configuration {
    apiKey;
    identifyEndpoint;
    computeEndpoint;
    coordinationEndpoint;
    autoRenew;
    accessToken;
    constructor(configValues) {
        this.apiKey = configValues.apiKey;
        this.identifyEndpoint = configValues.identifyEndpoint || SEAPLANE_IDENTIFY_API_ENDPOINT;
        this.computeEndpoint = configValues.computeEndpoint || SEAPLANE_COMPUTE_API_ENDPOINT;
        this.coordinationEndpoint = configValues.coordinationEndpoint || SEAPLANE_COORDINATION_API_ENDPOINT;
        this.autoRenew = configValues.autoRenew || true;
        this.accessToken = undefined;
    }
    values() {
        return {
            apiKey: this.apiKey,
            identifyEndpoint: this.identifyEndpoint,
            computeEndpoint: this.computeEndpoint,
            coordinationEndpoint: this.coordinationEndpoint,
            autoRenew: this.autoRenew
        };
    }
}
