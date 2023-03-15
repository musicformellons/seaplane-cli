from typing import Any, Dict, List, Optional

import requests

from ..configuration import Configuration, config
from ..model.metadata import Key, KeyValue, KeyValuePage, to_key_value, to_key_value_page
from ..util import unwrap
from ..util.base64url import base64url_encode_from_bytes
from .api_http import headers
from .api_request import provision_req


class MetadataAPI:
    """
    Class for handle Metadata API calls.
    """

    def __init__(self, configuration: Configuration = config) -> None:
        self.url = f"{configuration.coordination_endpoint}/config"
        self.req = provision_req(configuration._token_api)

    def set(self, key_value: KeyValue) -> bool:
        """Adds a value to the store at the given key.

        Parameters
        ----------
        key_value : KeyValue
            key-value pair to be set, for example, key=foo/bar, value=hello

        Returns
        -------
        boolean
            Returns true if successful or it will raise an HTTPError otherwise.
        """
        url = f"{self.url}/base64:{base64url_encode_from_bytes(key_value.key)}"

        set_result = unwrap(
            self.req(
                lambda access_token: requests.put(
                    url,
                    data=base64url_encode_from_bytes(key_value.value),
                    headers=headers(access_token),
                )
            )
        )

        return bool(set_result == "Ok")  # mypy bug, mypy can't know the

    def get(self, key: Key) -> KeyValue:
        """Returns the key value pair associated with the set key.

        Parameters
        ----------
        key : Key
            The key from a key-value previously set, for example, key=foo/bar

        Returns
        -------
        KeyValue
            Returns KeyValue if successful or it will raise an HTTPError otherwise.
        """
        url = f"{self.url}/base64:{base64url_encode_from_bytes(key.key)}"

        return unwrap(
            self.req(lambda access_token: requests.get(url, headers=headers(access_token))).map(
                lambda key_value: to_key_value(key_value)
            )
        )

    def delete(self, key: Key) -> bool:
        """Deletes the key value pair at from a given key.

        Parameters
        ----------
        directory : Key
            The key from a key-value previously set, for example, key=foo/bar

        Returns
        -------
        boolean
            Returns true if successful or it will raise an HTTPError otherwise.
        """
        url = f"{self.url}/base64:{base64url_encode_from_bytes(key.key)}"

        delete_result = unwrap(
            self.req(lambda access_token: requests.delete(url, headers=headers(access_token)))
        )

        return bool(delete_result == "Ok")  # mypy bug, mypy can't know the

    def get_page(
        self,
        directory: Optional[Key] = None,
        next_key: Optional[Key] = None,
    ) -> KeyValuePage:
        """Returns a single page of key value pairs for the given directory,
        beginning with the `next_key` key.

        If no directory is given, the root directory is used.
        If no `next_key` is given, the range begins from the start.

        Parameters
        ----------
        directory : Optional[Key]
            The metadata key-value store supports the use of directories.
            Directories are important tools to set up region and provider restrictions
            for groups of key-value pairs.
        next_key : Optional[Key]
            If more pages are desired, perform another range request using
            the `next_key` value from the first request as the `next_key` value of
            the following request.

        Returns
        -------
        KeyValuePage
            Returns KeyValuePage if successful or it will raise an HTTPError otherwise.
        """

        _url = self.url

        if directory is not None:
            _url = f"{self.url}/base64:{base64url_encode_from_bytes(directory.key)}/"

        params: Dict[str, Any] = {}
        if next_key is not None:
            params["from"] = f"base64:{base64url_encode_from_bytes(next_key.key)}"

        return unwrap(
            self.req(
                lambda access_token: requests.get(
                    _url, params=params, headers=headers(access_token)
                )
            ).map(lambda key_value_range: to_key_value_page(key_value_range))
        )

    def get_all_pages(
        self,
        directory: Optional[Key] = None,
        next_key: Optional[Key] = None,
    ) -> List[KeyValue]:
        """Returns all key value pair for the given directory, from the `next_key` key onwards.
        May perform multiple requests.

        If no directory is given, the root directory is used.
        If no `next_key` is given, the range begins from the start.

        Parameters
        ----------
        directory : Optional[Key]
            The metadata key-value store supports the use of directories.
            Directories are important tools to set up region and provider restrictions
            for groups of key-value pairs.
        next_key : Optional[Key]
            It begins from next_key in advance until the last page,
            if you don't want to begin from the start.

        Returns
        -------
        List[KeyValue]
            Returns a key value pair list if successful or it will raise an HTTPError otherwise.
        """

        pages: List[KeyValue] = []
        _next_lock = next_key

        while True:
            page_result = self.get_page(directory, _next_lock)

            page: KeyValuePage = page_result
            pages.extend(page.key_value_pairs)

            if page.next_key is not None:
                _next_lock = page.next_key
            else:
                return pages
