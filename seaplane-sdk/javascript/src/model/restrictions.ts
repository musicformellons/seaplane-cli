import { Provider, mapToProvider } from './provider';
import { Region, mapToRegions } from './region';
import { Key, mapKey } from './metadata';
import { SeaplaneError } from './errors';

export enum SeaplaneApi {
  Locks = 'locks',
  Metadata = 'config',
}

export enum RestrictionState {
  Enforced = 'Enforced',
  Pending = 'Pending',
}

export type RestrictionDetails = {
  regionsAllowed: Region[];
  regionsDenied: Region[];
  providersAllowed: Provider[];
  providersDenied: Provider[];
};

export type Restriction = {
  api: SeaplaneApi;
  directory: Key;
  details: RestrictionDetails;
  state: RestrictionState;
};

export type RestrictionPage = {
  restrictions: Restriction[];
  nextApi: SeaplaneApi | null;
  nextKey: Key | null;
};

export type LockInfo = {
  ttl: number;
  clientId: string;
  ip: string;
};

interface RestrictionBody {
  directory: string;
  api: string;
  state: string;
  details: {
    regions_allowed: [string];
    regions_denied: [string];
    providers_allowed: [string];
    providers_denied: [string];
  };
}

export const mapToRestriction = (restrictionBody: any): Restriction => {
  const restriction = restrictionBody as RestrictionBody;

  const key = mapKey(restriction['directory'] as string);

  if (key == null) {
    throw new SeaplaneError('Directory must not be null');
  }

  return {
    api: seaplaneApi(restriction['api']),
    directory: key,
    details: mapToRestrictionDetails(restriction),
    state: mapState(restriction['state']),
  };
};

const mapToRestrictionDetails = (restriction: RestrictionBody): RestrictionDetails => ({
  regionsAllowed: mapToRegions(restriction['details']['regions_allowed']),
  regionsDenied: mapToRegions(restriction['details']['regions_denied']),
  providersAllowed: mapToProvider(restriction['details']['providers_allowed']),
  providersDenied: mapToProvider(restriction['details']['providers_denied']),
});

const mapToSeaplaneApi = (api?: string): SeaplaneApi | null => {
  if (!api) {
    return null;
  }

  return seaplaneApi(api);
};

export const mapToRestrictionPage = (restrictionPage: {
  restrictions: [];
  next_api: string;
  next_key: string;
}): RestrictionPage => ({
  restrictions: restrictionPage['restrictions'].map(mapToRestriction),
  nextApi: mapToSeaplaneApi(restrictionPage['next_api']),
  nextKey: mapKey(restrictionPage['next_key']),
});

const capitalize: (string: string) => string = (string) => string.charAt(0).toUpperCase() + string.slice(1);

const seaplaneApi: (api: string) => SeaplaneApi = (api) => (api == 'config' ? SeaplaneApi.Metadata : SeaplaneApi.Locks);
const mapState: (state: string) => RestrictionState = (state) =>
  state == 'enforced' ? RestrictionState.Enforced : RestrictionState.Pending;
