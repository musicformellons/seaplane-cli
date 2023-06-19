from typing import Dict, Optional

import requests
from returns.result import Failure, Result, Success

from ..logging import log
from ..model.errors import HTTPError
from ..util import unwrap
from .api_http import SDK_HTTP_ERROR_CODE, headers


class TokenAPI:
    """
    Manage access token.

    Seaplane Python SDK manages the token by default,
    It can be managed manually as well.

    Any configuration change to the default `config` module,
    It'll reset `TokenAPI` local configurations, and renewed tokens.
    """

    def __init__(self, configuration) -> None:  # type: ignore
        self.url = f"{configuration.identify_endpoint}/token"
        self.api_key = configuration.seaplane_api_key
        self.access_token = configuration._current_access_token
        self.auto_renew = configuration._token_auto_renew
        self.tenant: Optional[str] = None

    def set_url(self, url: str) -> None:
        self.url = url

    def set_token(self, access_token: Optional[str]) -> None:
        """Set a valid Seaplane Token.

        Setting the token, will change auto-renew to False
        needing to renew the token manually when the token expires.

            $ from seaplane import sea

            $ token = sea.auth.get_token()
            $ sea.auth.set_token(token)

        If the access_token is None, Auto-renew will still False.

        Parameters
        ----------
        access_token : Optional[str]
        """
        self.auto_renew = False
        self.access_token = access_token

    def renew_token(self) -> str:
        """Renew the token.

        Any configuration change to the default `config` module,
        It'll the renewed token, and It's needed to renew the token again.

        Returns
        -------
        str
            Returns the renewed token.
        """
        token = self.get_token()
        self.access_token = token
        return token

    def get_token(self) -> str:
        """Request a new token.

        Returns
        -------
        str
            Returns the token.
        """
        return unwrap(self._request_access_token())["token"]

    def get_tenant(self) -> str:
        """Request the current tenant

        Returns
        -------
        str
            Returns the tenant.
        """
        return self.tenant if self.tenant else unwrap(self._request_access_token())["tenant"]

    def _request_access_token(self) -> Result[Dict[str, str], HTTPError]:
        try:
            log.info("Requesting access token...")
            if not self.api_key:
                log.error("API KEY not set, use sea.config.set_api_key")

            response = requests.post(self.url, json={}, headers=headers(self.api_key))

            if response.ok:
                token = response.json()
                self.access_token = token["token"]
                self.tenant = token["tenant"]
                return Success(token)
            else:
                self._current_access_token = None
                error_body = response.text
                log.error(
                    f"Bad Access token request code {response.status_code}, error {error_body}"
                )
                return Failure(HTTPError(response.status_code, error_body))

        except requests.exceptions.RequestException as err:
            self._current_access_token = None
            log.error(f"Request access token exception: {str(err)}")
            return Failure(HTTPError(SDK_HTTP_ERROR_CODE, str(err)))
