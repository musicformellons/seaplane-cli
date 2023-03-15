from enum import Enum
from typing import Any, Dict, List, NamedTuple, Optional

from ..errors import SeaplaneError
from ..metadata import Key, _to_key
from ..provider import Provider, to_providers
from ..region import Region, to_regions


class SeaplaneApi(Enum):
    """
    API enum for which data placement can be restricted.


    Attributes
        ----------
        locks: str
            Locks API data placement that can be restricted.
        metadata: str
            Metadata API data placement that can be restricted.
    """

    locks = "Locks"
    metadata = "Config"

    def __str__(self) -> str:
        return str(self.value)


class RestrictionState(Enum):
    """
    State of a restriction.


    Attributes
        ----------
        enforced: str
            Restriction is in effect.
        pending: str
            Restriction is being applied.
    """

    enforced = "Enforced"
    pending = "Pending"

    def __str__(self) -> str:
        return str(self.value)


class RestrictionDetails(NamedTuple):
    """
    Allow / deny lists of cloud providers and geographic regions
    to be associated with a set of records.

    Attributes
        ----------
        regions_allowed: str
            List of Regions which are allowed.
        regions_denied: str
            List of Regions which are denied.
        providers_allowed: str
            List of Providers which are allowed.
        providers_denied: str
            List of Providers which are denied.
    """

    regions_allowed: List[Region]
    regions_denied: List[Region]
    providers_allowed: List[Provider]
    providers_denied: List[Provider]


class Restriction(NamedTuple):
    """
    The provider and region restrictions information.

    Attributes
        ----------
        api: SeaplaneApi
            An API for which data placement can be restricted.
        directory: bytes
            Key used to select a key-value pair or a directory
            in configuration API.
        details: RestrictionDetails
            Allow / deny lists of cloud providers and geographic
            regions to be associated with a set of records.
        state: RestrictionState
            State of a restriction.
    """

    api: SeaplaneApi
    directory: Key
    details: RestrictionDetails
    state: RestrictionState


class RestrictionPage(NamedTuple):
    """
    Restriction Page class.

    It contains a paginated list of restrictions, next_key is used for the next call.

    Attributes
        ----------
        restrictions : List[Restriction]
            list of restrictions.
        next_key: Optional[Name]
            If next_key is non-null and you want to get the next RestrictionPage,
            you can repeat a query using next_key to continue getting RestrictionPage.
    """

    restrictions: List[Restriction]
    next_api: Optional[SeaplaneApi]
    next_key: Optional[Key]


def to_restriction(restriction: Dict[str, Any]) -> Restriction:
    key = _to_key(restriction["directory"])
    if key is None:
        raise SeaplaneError("Directory must not be null")

    return Restriction(
        api=SeaplaneApi(restriction["api"]),
        directory=key,
        details=_to_restriction_details(restriction["details"]),
        state=RestrictionState(restriction["state"]),
    )


def _to_restriction_details(restriction: Dict[str, Any]) -> RestrictionDetails:
    return RestrictionDetails(
        providers_allowed=to_providers(restriction["providers_allowed"]),
        providers_denied=to_providers(restriction["providers_denied"]),
        regions_allowed=to_regions(restriction["regions_allowed"]),
        regions_denied=to_regions(restriction["regions_denied"]),
    )


def _to_seaplane_api(api: Optional[str]) -> Optional[SeaplaneApi]:
    if api is None:
        return None

    return SeaplaneApi(api.capitalize())


def to_restriction_page(restriction_page: Dict[str, Any]) -> RestrictionPage:
    return RestrictionPage(
        restrictions=[
            to_restriction(restriction) for restriction in restriction_page["restrictions"]
        ],
        next_api=_to_seaplane_api(restriction_page["next_api"]),
        next_key=_to_key(restriction_page["next_key"]),
    )
