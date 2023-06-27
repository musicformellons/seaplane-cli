import os
from typing import Dict, Optional

from .api.token_api import TokenAPI
from .logging import log
from .model.errors import SeaplaneError

_SEAPLANE_COMPUTE_API_ENDPOINT = "https://compute.cplane.cloud/v2beta"
_SEAPLANE_COORDINATION_API_ENDPOINT = "https://metadata.cplane.cloud/v1"
_SEAPLANE_IDENTIFY_API_ENDPOINT = "https://flightdeck.cplane.cloud/v1"
_SEAPLANE_GLOBAL_SQL_API_ENDPOINT = "https://sql.cplane.cloud/v1"
_SEAPLANE_CARRIER_API_ENDPOINT = "https://carrier.cplane.cloud/v1"
_SEAPLANE_SUBSTATION_API_ENDPOINT = "https://substation.dev.cplane.cloud/v1"

_SEAPLANE_ENV_VAR_PRODUCTION = "SEAPLANE_APPS_PRODUCTION"

api_key_names = [
    "SEAPLANE_API_KEY",
    "OPENAI_API_KEY",
    "REPLICATE_API_KEY",
    "PINECONE_API_KEY",
    "PINECONE_API_ENV",
]


def get_env(key: str) -> Optional[str]:
    env = os.getenv(key)

    if not env:
        return None
    else:
        return env


def get_api_keys() -> Dict[str, str]:
    api_keys = {}

    for api_key_env_name in api_key_names:
        api_key = get_env(api_key_env_name)
        if api_key:
            api_keys[api_key_env_name] = api_key

    return api_keys


class Configuration:
    """
    Seaplane SDK Configuration.

    Everytime the configuration is changed,
    It'll clear local configurations to the default Auth module.
    """

    def __init__(self) -> None:
        self.seaplane_api_key: Optional[str] = None
        self.identify_endpoint = _SEAPLANE_IDENTIFY_API_ENDPOINT
        self.compute_endpoint = _SEAPLANE_COMPUTE_API_ENDPOINT
        self.coordination_endpoint = _SEAPLANE_COORDINATION_API_ENDPOINT
        self.global_sql_endpoint = _SEAPLANE_GLOBAL_SQL_API_ENDPOINT
        self.substation_endpoint = _SEAPLANE_SUBSTATION_API_ENDPOINT
        self.carrier_endpoint = _SEAPLANE_CARRIER_API_ENDPOINT
        self._current_access_token: Optional[str] = None
        self._token_auto_renew = True
        self.seaplane_api_key = get_env("SEAPLANE_API_KEY")
        self._api_keys: Dict[str, str] = get_api_keys()
        self._production = False
        self._update_token_api()

    def set_api_key(self, api_key: str) -> None:
        """Set the Seaplane API Key.

        The API Key is needed for the Seaplane Python SDK usage.

        Parameters
        ----------
        api_key : str
            Seaplane API Key.
        """
        self.seaplane_api_key = api_key
        self._update_token_api()

    def set_api_keys(self, api_keys: Dict[str, str]) -> None:
        """Set the Seaplane API Keys for Apps.

        The API Keys is needed for some of the Tasks.

        Supported Tasks API Keys:

        Seaplane: SEAPLANE_API_KEY
        Open AI: OPENAI_API_KEY
        Replicate: REPLICATE_API_KEY

        For example, for use an OpenAI Task,
        you need to provide the Key - Value, of the API Key.

            $ from seaplane import sea

            $ api_keys = {"OPENAI_API_KEY": "sp-api-key-test" }
            $ sea.config.set_api_keys(api_keys)

        Parameters
        ----------
        api_keys : object
            API Keys and values.
        """
        self._api_keys = api_keys

        if api_keys is None:
            raise SeaplaneError("api_keys parameters can't be None")
        elif api_keys.get("SEAPLANE_API_KEY", None) is not None:
            self.seaplane_api_key = api_keys["SEAPLANE_API_KEY"]

        self._update_token_api()

    def set_token(self, access_token: Optional[str]) -> None:
        """Set a valid Seaplane Token globally.

        The access token will be persisted even if any configuration changes.

        Setting the token, will change auto-renew to False
        needing to renew the token manually when the token expires.

            $ from seaplane import sea

            $ token = sea.auth.get_token()
            $ sea.config.set_token(token)

        If the access_token is None, Auto-renew will still False.

        Parameters
        ----------
        access_token : Optional[str]
        """
        self._current_access_token = access_token
        self._token_auto_renew = False
        self._token_api.set_token(access_token)

        log.info("Set access token, Auto-Renew deactivated")

    def token_autorenew(self, autorenew: bool) -> None:
        """Changes Auto-renew state globally.

        If Auto-renew is True will automatically renew the actual token
        when the previous token expires. Auto-renew is True by default.

        Setting Auto-renew to False will get a token the first call,
        once the token expires, It throws an HTTPError with a 401 http status code
        until the token is renew it calling `sea.auth.renew_token()`.

            $ from seaplane import sea

            $ sea.config.token_autorenew(False)
            $ ... When the token expires, renew it ...
            $ sea.auth.renew_token()

        Parameters
        ----------
        autorenew : bool
            True to activate Auto-renew, False to deactivate Auto-renew.
        """
        self._token_auto_renew = autorenew
        self._current_access_token = None
        self._update_token_api()

        log.info(f"Auto-Renew to {autorenew}")

    def set_compute_endpoint(self, endpoint: str) -> None:
        if endpoint[-1] == "/":
            self.compute_endpoint = endpoint.rstrip(endpoint[-1])
        else:
            self.compute_endpoint = endpoint

        self._update_token_api()

    def set_coordination_endpoint(self, endpoint: str) -> None:
        if endpoint[-1] == "/":
            self.coordination_endpoint = endpoint.rstrip(endpoint[-1])
        else:
            self.coordination_endpoint = endpoint

        self._update_token_api()

    def set_identify_endpoint(self, endpoint: str) -> None:
        if endpoint[-1] == "/":
            self.identify_endpoint = endpoint.rstrip(endpoint[-1])
        else:
            self.identify_endpoint = endpoint

        self._update_token_api()

    def set_global_sql_endpoint(self, endpoint: str) -> None:
        if endpoint[-1] == "/":
            self.global_sql_endpoint = endpoint.rstrip(endpoint[-1])
        else:
            self.global_sql_endpoint = endpoint

        self._update_token_api()

    def set_substation_endpoint(self, endpoint: str) -> None:
        if endpoint[-1] == "/":
            self.substation_endpoint = endpoint.rstrip(endpoint[-1])
        else:
            self.substation_endpoint = endpoint

        self._update_token_api()

    def set_carrier_endpoint(self, endpoint: str) -> None:
        if endpoint[-1] == "/":
            self.carrier_endpoint = endpoint.rstrip(endpoint[-1])
        else:
            self.carrier_endpoint = endpoint

        self._update_token_api()

    def _update_token_api(self) -> None:
        self._token_api = TokenAPI(self)

    def log_level(self, level: int) -> None:
        """Change logging level.

        Seaplane uses Python logging module for internal logs.
        Python logging levels can be used directly with Seaplane Python SDK or
        use the already defined in seaplane.log module.

            $ from seaplane import sea, log
            $ sea.config.log_level(log.INFO)


        Parameters
        ----------
        level : int
            Logging Level from Python logging module,
            like DEBUG, INFO, WARNING, ERROR, CRITICAL
        """
        log.level(level)

        if level == log.DEBUG:
            log.debug("Seaplane debug activated")
            log.debug(f"Identify endpoint: {self.identify_endpoint}")
            log.debug(f"Compute endpoint: {self.compute_endpoint}")
            log.debug(f"Coordination endpoint: {self.coordination_endpoint}")

    def log_enable(self, enable: bool) -> None:
        """Enable or disable the Seaplane logging for the SDK.

        Parameters
        ----------
        enable : bool
            True to enable, False to disable.
        """
        if enable:
            log.enable()
        else:
            log.disable()

    def is_production(self) -> bool:
        if not self._production:
            return os.getenv(_SEAPLANE_ENV_VAR_PRODUCTION, "").lower() == "true"

        return self._production

    def set_production(self, is_production: bool) -> None:
        self._production = is_production


config = Configuration()
