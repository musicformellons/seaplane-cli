from typing import Any, Dict, List, Optional

import requests

from ..configuration import Configuration, config
from ..model.errors import SeaplaneError
from ..model.metadata import Key
from ..model.restrict import (
    Restriction,
    RestrictionDetails,
    RestrictionPage,
    SeaplaneApi,
    to_restriction,
    to_restriction_page,
)
from ..util import unwrap
from ..util.base64url import base64url_encode_from_bytes
from .api_http import headers
from .api_request import provision_req


class RestrictAPI:
    """
    Class for handle Restrict API calls.
    """

    def __init__(self, configuration: Configuration = config) -> None:
        self.url = f"{configuration.coordination_endpoint}/restrict"
        self.req = provision_req(configuration._token_api)

    def get(self, api: SeaplaneApi, key: Key) -> Restriction:
        """Returns restriction details for an API-directory combination

        Parameters
        ----------
        api: SeaplaneApi
            The name of a Seaplane API,
              for example "SeaplaneApi.metadata" or "SeaplaneApi.locks"
        key: Key
            Key pointing to a directory.

        Returns
        -------
        Restriction
            Returns Restriction if successful or it will raise an HTTPError otherwise.
        """
        _url = f"{self.url}/{api}/base64:{base64url_encode_from_bytes(key.key)}/"

        return unwrap(
            self.req(lambda access_token: requests.get(_url, headers=headers(access_token))).map(
                lambda restriction_response: to_restriction(restriction_response)
            )
        )

    def set(self, api: SeaplaneApi, key: Key, restriction_details: RestrictionDetails) -> bool:
        """Sets a restriction for an API-directory combination

        Parameters
        ----------
        api: SeaplaneApi
            The name of a Seaplane API,
              for example "SeaplaneApi.metadata" or "SeaplaneApi.locks"
        key: Key
            Key pointing to a directory.
        restriction_details: RestrictionDetails
            Allow or deny lists for geographies and cloud providers
            to be associated with a given directory.

        Returns
        -------
        Restriction
            Returns Restriction if successful or it will raise an HTTPError otherwise.
        """
        _url = f"{self.url}/{api}/base64:{base64url_encode_from_bytes(key.key)}/"

        _payload = {
            "regions_allowed": [str(region) for region in restriction_details.regions_allowed],
            "regions_denied": [str(region) for region in restriction_details.regions_denied],
            "providers_allowed": [
                str(provider) for provider in restriction_details.providers_allowed
            ],
            "providers_denied": [
                str(provider) for provider in restriction_details.providers_denied
            ],
        }

        set_result = unwrap(
            self.req(
                lambda access_token: requests.put(
                    _url, json=_payload, headers=headers(access_token)
                )
            )
        )

        return bool(set_result == "Ok")  # mypy bug, mypy can't know the type

    def delete(
        self,
        api: SeaplaneApi,
        key: Key,
    ) -> bool:
        """Removes a restriction for an API-directory combination

        Parameters
        ----------
        api: SeaplaneApi
            The name of a Seaplane API,
              for example "SeaplaneApi.metadata" or "SeaplaneApi.locks"
        key: Key
            Key pointing to a directory.

        Returns
        -------
        boolean
            Returns true if successful or it will raise an HTTPError otherwise.
        """
        url = f"{self.url}/{api}/base64:{base64url_encode_from_bytes(key.key)}/"

        delete_result = unwrap(
            self.req(lambda access_token: requests.delete(url, headers=headers(access_token)))
        )

        return bool(delete_result == "Ok")  # mypy bug, mypy can't know the

    def get_page(
        self,
        api: Optional[SeaplaneApi] = None,
        from_restriction: Optional[Key] = None,
        is_all_range: bool = False,
    ) -> RestrictionPage:
        """Returns a single page of restriction information for the given Seaplane API,
        beginning with the `from_restriction` key.

        If more pages are desired, perform another page request using the
        `next_api` and `next_key` values from the first `RestrictionPage` request as the
        `api` and `from_restriction` values of the following request, or use `get_all_pages`.


        Parameters
        ----------
        api: Optional[SeaplaneApi]
            The name of a Seaplane API,
              for example "SeaplaneApi.metadata" or "SeaplaneApi.locks"
        from_restriction : Optional[Key]
            If more pages are desired, perform another range request using
            the `from_restriction` value from the first request as the `from_restriction` value of
            the following request. If you use `from_restriction` is mandatory to indicate
            the `api` as well.
        is_all_range: bool
            Set to True to get all Restrictions from any Seaplane API, It filters by API otherwise.
            By default `is_all_range` is False. If you pass `api` parameter
            will be used to filter by API.

        Returns
        -------
        RestrictionPage
            Returns RestrictionPage if successful or it will raise an HTTPError otherwise.
        """

        _url = self.url

        if api is not None and not is_all_range:
            _url = f"{self.url}/{api}"

        params: Dict[str, Any] = {}
        if from_restriction is not None:
            params["from"] = f"base64:{base64url_encode_from_bytes(from_restriction.key)}"
            params["from_api"] = str(api)
            if not api:
                raise SeaplaneError("You must set api with from_restriction parameters.")

        return unwrap(
            self.req(
                lambda access_token: requests.get(
                    _url, params=params, headers=headers(access_token)
                )
            ).map(lambda lock_range: to_restriction_page(lock_range))
        )

    def get_all_pages(
        self,
        api: Optional[SeaplaneApi] = None,
        from_restriction: Optional[Key] = None,
    ) -> List[Restriction]:
        """
        Returns restrictions for the given Seaplane API,
        from the `from_restriction` key onwards. May perform multiple requests.

        If no API is given, it will return all API restrictions.
        If no `from_restriction` is given, the range begins from the start.

        Parameters
        ----------
        api: Optional[SeaplaneApi]
            The name of a Seaplane API,
              for example "SeaplaneApi.metadata" or "SeaplaneApi.locks"
        from_restriction : Optional[Key]
            It begins from a restriction in advance until the last page,
            if you don't want to begin from the start.

        Returns
        -------
        List[Restriction]
            Returns a List of Restrictions if successful or it will raise an HTTPError otherwise.
        """

        pages: List[Restriction] = []
        _from_restriction = from_restriction
        _from_api = api

        while True:
            page_result = self.get_page(_from_api, _from_restriction, is_all_range=True)

            page: RestrictionPage = page_result
            pages.extend(page.restrictions)

            if page.next_key is not None and page.next_api is not None:
                _from_restriction = page.next_key
                _from_api = page.next_api
            else:
                return pages
