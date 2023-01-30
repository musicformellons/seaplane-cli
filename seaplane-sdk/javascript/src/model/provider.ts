export enum Provider {
  AWS = 'AWS',
  Azure = 'AZURE',
  DigitalOcean = 'DIGITALOCEAN',
  Equinix = 'EQUINIX',
  GCP = 'GCP',
}
const mapProvider = (providerString: string): Provider => {
  let provider = Provider.AWS;

  switch (providerString) {
    case 'AWS': {
      provider = Provider.AWS;
      break;
    }
    case 'AZURE': {
      provider = Provider.Azure;
      break;
    }
    case 'DIGITALOCEAN': {
      provider = Provider.DigitalOcean;
      break;
    }
    case 'EQUINIX': {
      provider = Provider.Equinix;
      break;
    }
    case 'GCP': {
      provider = Provider.GCP;
      break;
    }
  }

  return provider;
};
export const mapToProvider = (providers?: string[]): Provider[] => {
  if (!providers) {
    return [];
  }

  return providers.map((provider) => mapProvider(provider));
};
