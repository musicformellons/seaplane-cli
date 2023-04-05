import requests

from ..configuration import Configuration, config
from ..model.sql import CreatedDatabase, to_created_database
from ..util import unwrap
from .api_http import headers
from .api_request import provision_req


class GlobalSQL:
    """
    Class for handle Global SQL API calls.
    """

    def __init__(self, configuration: Configuration = config) -> None:
        self.url = f"{configuration.global_sql_endpoint}/databases"
        self.req = provision_req(configuration._token_api)

    def create_database(self) -> CreatedDatabase:
        """Create a new Global Seaplane Database.

        Returns
        -------
        CreatedDatabase
            Returns a CreatedDatabase if successful or it will raise an HTTPError otherwise.
        """

        return unwrap(
            self.req(
                lambda access_token: requests.post(
                    self.url, data="{}", headers=headers(access_token)
                )
            ).map(lambda database: to_created_database(database))
        )
