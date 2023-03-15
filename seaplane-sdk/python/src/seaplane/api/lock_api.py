from typing import Any, Dict, List, Optional

import requests

from ..configuration import Configuration, config
from ..model.locks import HeldLock, Lock, LockPage, Name, to_held_lock, to_lock, to_lock_page
from ..util import unwrap
from ..util.base64url import base64url_encode_from_bytes
from .api_http import headers
from .api_request import provision_req


class LockAPI:
    """
    Class for handle Lock API calls.
    """

    def __init__(self, configuration: Configuration = config) -> None:
        self.url = f"{configuration.coordination_endpoint}/locks"
        self.req = provision_req(configuration._token_api)

    def get(self, name: Name) -> Lock:
        """Gets information about a single lock.

        Parameters
        ----------
        name : Name
            Name of the lock.

        Returns
        -------
        Lock
            Returns Lock if successful or it will raise an HTTPError otherwise.
        """
        _url = f"{self.url}/base64:{base64url_encode_from_bytes(name.name)}"

        return unwrap(
            self.req(lambda access_token: requests.get(_url, headers=headers(access_token))).map(
                lambda lock_response: to_lock(lock_response)
            )
        )

    def acquire(self, name: Name, client_id: str, ttl: int) -> HeldLock:
        """Attempts to acquire the lock with the given lock name with the given TTL.
        Client-ID should identify the client making the request for debugging purposes.

        Parameters
        ----------
        name : Name
            Name of the lock.
        client_id : Name
            client_id is an identifier showing who is currently holding the lock.
        ttl: int
            This is the requested time to live, in seconds.

        Returns
        -------
        HeldLock
            Returns HeldLock if successful or it will raise an HTTPError otherwise.
        """
        _url = f"{self.url}/base64:{base64url_encode_from_bytes(name.name)}"

        params: Dict[str, Any] = {}
        params["client-id"] = client_id
        params["ttl"] = ttl

        return unwrap(
            self.req(
                lambda access_token: requests.post(
                    _url, params=params, headers=headers(access_token)
                )
            ).map(lambda held_lock_response: to_held_lock(held_lock_response))
        )

    def release(self, name: Name, id: str) -> bool:
        """Attempts to release the given lock.

        Parameters
        ----------
        name : Name
            Name of the lock.
        id: str
            ID which allows for the renewal/release of a lock, represents "holding"
            the lock, this is usually only seen by the original acquirer of the lock
            but may be leaked by listing the locks.

        Returns
        -------
        bool
            Returns bool if successful or it will raise an HTTPError otherwise.
        """
        _url = f"{self.url}/base64:{base64url_encode_from_bytes(name.name)}"

        params: Dict[str, Any] = {}
        params["id"] = id

        release_result = unwrap(
            self.req(
                lambda access_token: requests.delete(
                    _url, params=params, headers=headers(access_token)
                )
            )
        )

        return bool(release_result == "OK")  # mypy bug, mypy can't know the

    def renew(self, name: Name, id: str, ttl: int) -> bool:
        """Attempts to renew the given lock.

        Parameters
        ----------
        name : Name
            Name of the lock.
        id: str
            ID which allows for the renewal/release of a lock, represents "holding"
            the lock, this is usually only seen by the original acquirer of the lock
            but may be leaked by listing the locks.
        ttl: int
            This is the requested time to live, in seconds.

        Returns
        -------
        bool
            Returns bool if successful or it will raise an HTTPError otherwise.
        """
        _url = f"{self.url}/base64:{base64url_encode_from_bytes(name.name)}"

        params: Dict[str, Any] = {}
        params["id"] = id
        params["ttl"] = ttl

        renew_result = unwrap(
            self.req(
                lambda access_token: requests.patch(
                    _url, params=params, headers=headers(access_token)
                )
            )
        )

        return bool(renew_result == "OK")

    def get_page(
        self,
        directory: Optional[Name] = None,
        from_lock: Optional[Name] = None,
    ) -> LockPage:
        """Returns a single page of lock information for the given directory,
        beginning with the `from_lock` key.

        If no directory is given, the root directory is used.
        If no `from_lock` is given, the range begins from the start.

        Parameters
        ----------
        directory : Optional[Name]
            The metadata key-value store supports the use of directories.
            Directories are important tools to set up region and provider restrictions
            for groups of key-value pairs.
        from_lock : Optional[Name]
            If more pages are desired, perform another range request using
            the `from_lock` value from the first request as the `from_lock` value of
            the following request.

        Returns
        -------
        LockPage
            Returns LockPage if successful or it will raise an HTTPError otherwise.
        """

        _url = self.url

        if directory is not None:
            _url = f"{self.url}/base64:{base64url_encode_from_bytes(directory.name)}/"

        params: Dict[str, Any] = {}
        if from_lock is not None:
            params["from"] = f"base64:{base64url_encode_from_bytes(from_lock.name)}"

        return unwrap(
            self.req(
                lambda access_token: requests.get(
                    _url, params=params, headers=headers(access_token)
                )
            ).map(lambda lock_range: to_lock_page(lock_range))
        )

    def get_all_pages(
        self,
        directory: Optional[Name] = None,
        from_lock: Optional[Name] = None,
    ) -> List[Lock]:
        """Returns all held lock information for the given directory,
        from the `from_lock` key onwards. May perform multiple requests.

        If no directory is given, the root directory is used.
        If no `from_lock` is given, the range begins from the start.

        Parameters
        ----------
        directory : Optional[Name]
            The metadata key-value store supports the use of directories.
            Directories are important tools to set up region and provider restrictions
            for groups of key-value pairs.
        from_lock : Optional[Name]
            It begins from from_lock in advance until the last page,
            if you don't want to begin from the start.

        Returns
        -------
        List[Lock]
            Returns a List of Locks if successful or it will raise an HTTPError otherwise.
        """

        pages: List[Lock] = []
        _from_lock = from_lock

        while True:
            page_result = self.get_page(directory, _from_lock)

            page: LockPage = page_result
            pages.extend(page.locks)

            if page.next_lock is not None:
                _from_lock = page.next_lock
            else:
                return pages
