from typing import Any, List, Text

import requests
from returns.result import Result

from ..configuration import Configuration, config
from ..model.compute.active_configuration import ActiveConfiguration
from ..model.compute.formation_configuration import FormationConfiguration, to_formation_config
from ..model.errors import HTTPError
from .api_http import headers, to_json
from .api_request import provision_req


class FormationConfigurationAPI:
    """
    Class for handle Configuration and Active Configuration API calls.
    Links:
      - https://developers.seaplane.io/reference/get_formations-formationname-configurations
      - https://developers.seaplane.io/reference/get_formations-formationname-activeconfiguration
    """

    def __init__(self, configuration: Configuration = config) -> None:
        self.url = f"{configuration.compute_endpoint}/formations"
        self.req = provision_req(configuration._token_api)

    def create(
        self, formation_name: str, formation: FormationConfiguration, active: bool = False
    ) -> Result[str, HTTPError]:
        url = f"{self.url}/{formation_name}/configurations"
        payload = to_json(formation)
        params = {"active": active}

        return self.req(
            lambda access_token: requests.post(
                url=url, json=payload, params=params, headers=headers(access_token)
            )
        )

    def get_all(self, formation_name: Text) -> Result[List[str], HTTPError]:
        url = f"{self.url}/{formation_name}/configurations"
        return self.req(lambda access_token: requests.get(url, headers=headers(access_token)))

    def get(self, formation_name: Text, id: Text) -> Result[FormationConfiguration, HTTPError]:
        url = f"{self.url}/{formation_name}/configurations/{id}"

        return self.req(lambda access_token: requests.get(url, headers=headers(access_token))).map(
            lambda json: to_formation_config(json)
        )

    def delete(self, formation_name: Text, id: Text) -> Result[Any, HTTPError]:
        url = f"{self.url}/{formation_name}/configurations/{id}"

        return self.req(lambda access_token: requests.delete(url, headers=headers(access_token)))

    def get_active_config(self, formation_name: Text) -> Result[Any, HTTPError]:
        url = f"{self.url}/{formation_name}/activeConfiguration"

        return self.req(lambda access_token: requests.get(url, headers=headers(access_token)))

    def set_active_config(
        self, formation_name: Text, active_configuration: ActiveConfiguration, force: bool
    ) -> Result[Any, HTTPError]:
        url = f"{self.url}/{formation_name}/activeConfiguration"
        params = {"force": force}
        payload = active_configuration.__dict__

        return self.req(
            lambda access_token: requests.put(
                url, headers=headers(access_token), params=params, json=payload
            )
        )

    def stop_formation(self, formation_name: Text) -> Result[Any, HTTPError]:
        url = f"{self.url}/{formation_name}/activeConfiguration"

        return self.req(lambda access_token: requests.delete(url, headers=headers(access_token)))
