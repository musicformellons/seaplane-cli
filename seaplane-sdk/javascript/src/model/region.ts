export enum Region {
  Asia = 'XA',
  RepublicOfChina = 'XC',
  Europe = 'XE',
  Africa = 'XF',
  NorthAmerica = 'XN',
  Oceania = 'XO',
  Antartica = 'XQ',
  SouthAmerica = 'XS',
  Uk = 'XU',
}

const mapRegion = (regionString: string): Region => {
  let region = Region.NorthAmerica;

  switch (regionString) {
    case 'XA': {
      region = Region.Asia;
      break;
    }
    case 'XC': {
      region = Region.RepublicOfChina;
      break;
    }
    case 'XE': {
      region = Region.Europe;
      break;
    }
    case 'XF': {
      region = Region.Africa;
      break;
    }
    case 'XN': {
      region = Region.NorthAmerica;
      break;
    }
    case 'XO': {
      region = Region.Oceania;
      break;
    }
    case 'XQ': {
      region = Region.Antartica;
      break;
    }
    case 'XS': {
      region = Region.SouthAmerica;
      break;
    }
    case 'XU': {
      region = Region.Uk;
      break;
    }
  }

  return region;
};

export const mapToRegions = (regions?: string[]): Region[] => {
  if (!regions) {
    return [];
  }

  return regions.map((region) => mapRegion(region));
};
