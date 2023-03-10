export type Flight = {
  oid: string;
  name: string;
  image: string;
  status?: string | null;
};

export type Formation = {
  oid: string;
  name: string;
  url?: string | null;
  flights: Flight[];
  gateway_flight?: string | null;
};

export type MetaPage = {
  total: number;
  next: string;
  prev: string;
};

export type FormationPage = {
  formations: Formation[];
  meta: MetaPage;
};

export const toFormationPage = (formationPage: any) => ({
  // eslint-disable-line
  formations: toFormations(formationPage.formations),
  meta: {
    total: formationPage.meta.total,
    next: formationPage.meta.next,
    prev: formationPage.meta.prev,
  },
});

export const toFormation = (formation: any) => ({
  // eslint-disable-line
  oid: formation.oid,
  name: formation.name,
  url: formation.url,
  flights: formation.flights,
  gateway_flight: formation['gateway-flight'],
});

export const toFormations = (
  formations: [any], // eslint-disable-line
) => formations.map((formation) => toFormation(formation));
