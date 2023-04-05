from typing import Any, Dict, List, NamedTuple, Optional


class Flight(NamedTuple):
    """
    Flight class.
    """

    oid: str
    name: str
    image: str
    status: Optional[str] = None


def to_flights(flights: List[Dict[str, Any]]) -> List[Flight]:
    return [Flight(**flight) for flight in flights]


class Formation(NamedTuple):
    """
    Formation class.
    """

    oid: str
    name: str
    url: Optional[str] = None
    flights: List[Flight] = []
    gateway_flight: Optional[str] = None


class MetaPage(NamedTuple):
    total: int
    next: str
    prev: str


class FormationPage(NamedTuple):
    formations: List[Formation]
    meta: MetaPage


def to_formation_page(formation_page: Dict[str, Any]) -> FormationPage:
    return FormationPage(
        formations=to_formations(formation_page["objects"]),
        meta=MetaPage(**formation_page["meta"]),
    )


def to_formation(formation: Dict[str, Any]) -> Formation:
    gateway = formation.pop("gateway-flight")
    flights = to_flights(formation.pop("flights"))
    formation_modified = {**formation, "flights": flights, "gateway_flight": gateway}
    return Formation(**formation_modified)


def to_formations(formations: List[Dict[str, Any]]) -> List[Formation]:
    return [to_formation(formation) for formation in formations]
