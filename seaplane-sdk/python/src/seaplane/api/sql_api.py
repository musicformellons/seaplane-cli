from typing import List

import requests

from ..configuration import Configuration, config
from ..model.sql import CreatedDatabase, to_created_database, to_list_databases
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

    def list_databases(self) -> List[str]:
        """List all Global Seaplane Databases.

        Returns
        -------
        list[database_name: str]
            Returns a list of database names if successful or it will raise an HTTPError otherwise.
        """

        return unwrap(
            self.req(
                lambda access_token: requests.get(self.url, headers=headers(access_token))
            ).map(lambda databases: to_list_databases(databases))
        )
