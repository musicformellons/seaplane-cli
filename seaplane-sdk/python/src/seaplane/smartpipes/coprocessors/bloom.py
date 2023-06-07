from typing import Any, Callable, Dict, Optional, Tuple

import requests

from ...api.api_http import headers
from ...api.api_request import provision_req
from ...configuration import config
from ...logging import log
from ...model.errors import SeaplaneError
from ...util import unwrap

SEAPLANE_API_KEY_NAME = "SEAPLANE_API_KEY"


def _check_seaplane_api_key() -> None:
    if config.seaplane_api_key is None:
        raise SeaplaneError(
            f"Seaplane API Key `{SEAPLANE_API_KEY_NAME}` is not set,\
                  use `sea.config.set_api_key`."
        )


def bloom() -> Callable[[Dict[str, Any]], Any]:
    def model(input: Dict[str, Any]) -> Any:
        _check_seaplane_api_key()

        url = f"{config.substation_endpoint}/completions"
        req = provision_req(config._token_api)

        payload = {"prompt": input}

        return unwrap(
            req(
                lambda access_token: requests.post(
                    url,
                    json=payload,
                    headers=headers(access_token),
                )
            )
        )

    return model


class Bloom:
    def __init__(self, func: Callable[[Any], Any], id: str, model: Optional[str]) -> None:
        self.func = func
        self.args: Optional[Tuple[Any, ...]] = None
        self.kwargs: Optional[Dict[str, Any]] = None
        self.type = "inference"
        self.model = model
        self.id = id

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        if self.model == "bloom":
            log.info("Processing Bloom Model...")
            self.args = self.args + (bloom(),)

            return self.func(*self.args, **self.kwargs)

    def print(self) -> None:
        log.info(f"id: {self.id}, type: {self.type}, model: {self.model}")
