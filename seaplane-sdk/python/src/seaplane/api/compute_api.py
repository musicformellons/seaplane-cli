import requests

from ..configuration import Configuration, config
from ..model.compute import Formation, FormationPage, to_formation, to_formation_page
from ..util import unwrap
from .api_http import headers
from .api_request import provision_req


class ComputeAPI:
    """
    Class for handle Compute API calls.
    """

    def __init__(self, configuration: Configuration = config) -> None:
        self.url = f"{configuration.compute_endpoint}/formations"
        self.req = provision_req(configuration._token_api)

    def create(self, formation: Formation) -> None:
        """Create a new Formation and returns the IDs of the created Formation.

        Parameters
        ----------
        formation : Formation
            formation with flights to be created and deployed.
        """

        _payload = {
            "oid": formation.oid,
            "name": formation.name,
            "url": formation.url,
            "flights": [
                {"name": flight.name, "oid": flight.oid, "image": flight.image}
                for flight in formation.flights
            ],
            "gateway-flight": formation.gateway_flight,
        }

        unwrap(
            self.req(
                lambda access_token: requests.post(
                    self.url,
                    json=_payload,
                    headers=headers(access_token),
                )
            )
        )

    def get(self, formation_id: str) -> Formation:
        """Returns the Formation associated with the created Formation OID.

        Parameters
        ----------
        formation_id : str
            The id from a Formation previously created.

        Returns
        -------
        Formation
            Returns Formation if successful or it will raise an HTTPError otherwise.
        """
        url = f"{self.url}/{formation_id}"

        return unwrap(
            self.req(lambda access_token: requests.get(url, headers=headers(access_token))).map(
                lambda formation: to_formation(formation)
            )
        )

    def delete(self, formation_id: str) -> None:
        """Deletes the Formation from a given oid.

        Parameters
        ----------
        formation_id : str
            The oid from a Formation previously created.
        """
        url = f"{self.url}/{formation_id}"

        unwrap(self.req(lambda access_token: requests.delete(url, headers=headers(access_token))))

    def get_page(
        self,
    ) -> FormationPage:
        """Returns a list of all the Formations you have access to

        Returns
        -------
        List[Formations]
            Returns Formations and meta info for next and previous pages if successful
            or it will raise an HTTPError otherwise.
        """

        return unwrap(
            self.req(
                lambda access_token: requests.get(self.url, headers=headers(access_token))
            ).map(lambda formation_page: to_formation_page(formation_page))
        )
