from typing import Any, Callable, Dict, Optional, Tuple

import requests

from ...configuration import config
from ...logging import log
from ...model.errors import SeaplaneError

REPLICATE_URL = "https://api.replicate.com/v1"
_REPLICATE_API_KEY_NAME = "REPLICATE_API_KEY"


def _check_replicate_api_key() -> None:
    if config._api_keys is None:
        raise SeaplaneError(
            f"Replicate API Key `{_REPLICATE_API_KEY_NAME}` is not set,\
                  use `sea.config.set_api_keys`."
        )
    elif not config._api_keys.get(_REPLICATE_API_KEY_NAME, None):
        raise SeaplaneError(
            f"Replicate API Key `{_REPLICATE_API_KEY_NAME}` is not set,\
                  use `sea.config.set_api_keys`."
        )


def inference(version: str) -> Callable[[Dict[str, Any]], Any]:
    def model(input: Dict[str, Any]) -> Any:
        """
        Replicate input example for stable diffusion:

            "input": {
                "width": 512,
                "height": 512,
                "num_outputs": 4,
                "prompt": "prompt",
                "seed": 11,
            },
        """
        _check_replicate_api_key()

        headers = {"Authorization": f"Token {config._api_keys[_REPLICATE_API_KEY_NAME]}"}

        payload = {"version": version, "input": input}

        response = requests.post(
            f"{REPLICATE_URL}/predictions",
            json=payload,
            headers=headers,
        )

        if response.ok:
            result = response.json()
            id = result["id"]

            while not result.get("output", None):
                endpoint = f"{REPLICATE_URL}/predictions/{id}"
                response = requests.get(
                    endpoint,
                    headers=headers,
                )

                log.info("Loading...")
                if response.ok:
                    result = response.json()

                    if result.get("output", None) is not None:
                        return result
        else:
            return response.text

    return model


class Replicate:
    def __init__(
        self, func: Callable[[Any], Any], type: str, id: str, model: Optional[str]
    ) -> None:
        self.func = func
        self.args: Optional[Tuple[Any, ...]] = None
        self.kwargs: Optional[Dict[str, Any]] = None
        self.type = type
        self.model = model
        self.id = id

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        if self.model == "stable-diffusion":
            log.info("Accessing Replicate stable-diffusion task...")
            self.args = self.args + (
                inference("27b93a2413e7f36cd83da926f3656280b2931564ff050bf9575f1fdf9bcd7478"),
            )

            return self.func(*self.args, **self.kwargs)
        elif self.model:
            log.info(f"Accessing Replicate {self.model} task...")
            self.args = self.args + (inference(self.model),)

            return self.func(*self.args, **self.kwargs)

    def print(self) -> None:
        log.info(f"id: {self.id}, type: {self.type}, model: {self.model}")
