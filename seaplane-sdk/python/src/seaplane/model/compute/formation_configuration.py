from typing import Any, Dict, List, NamedTuple, Optional

from ..provider import Provider, to_providers
from ..region import Region, to_regions
from .flight import Flight, to_flights


class FormationConfiguration(NamedTuple):
    """
    Formation class with all formation attributes.
    """

    flights: List[Flight]
    affinity: Optional[List[str]] = None
    connections: Optional[List[str]] = None
    public_endpoints: Optional[object] = None
    formation_endpoints: Optional[object] = None
    flight_endpoints: Optional[object] = None
    providers_allowed: Optional[List[Provider]] = None
    providers_denied: Optional[List[Provider]] = None
    regions_allowed: Optional[List[Region]] = None
    regions_denied: Optional[List[Region]] = None


def to_formation_config(fconfig: Dict[str, Any]) -> FormationConfiguration:
    formation_config = fconfig.copy()
    formation_config.pop("flights", None)
    formation_config.pop("providers_allowed", None)
    formation_config.pop("providers_denied", None)
    formation_config.pop("regions_allowed", None)
    formation_config.pop("regions_denied", None)

    return FormationConfiguration(
        **formation_config,
        flights=to_flights(fconfig["flights"]),
        providers_allowed=to_providers(fconfig.pop("providers_allowed", None)),
        providers_denied=to_providers(fconfig.pop("providers_denied", None)),
        regions_allowed=to_regions(fconfig.pop("regions_allowed", None)),
        regions_denied=to_regions(fconfig.pop("regions_denied", None)),
    )
