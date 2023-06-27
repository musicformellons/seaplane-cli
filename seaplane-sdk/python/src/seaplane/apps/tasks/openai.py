from typing import Any, Callable, Dict, Optional, Tuple

import requests

from ...api.api_http import headers
from ...api.api_request import handle_request
from ...configuration import config
from ...logging import log
from ...model.errors import SeaplaneError
from ...util import unwrap

OPENAI_API_KEY_NAME = "OPENAI_API_KEY"
OPENAI_URL = "https://api.openai.com"


def _check_openai_api_keys() -> None:
    if config._api_keys is None:
        raise SeaplaneError(
            f"OPENAI API Key `{OPENAI_API_KEY_NAME}` is not set, \
                  use `sea.config.set_api_keys`."
        )
    elif not config._api_keys.get(OPENAI_API_KEY_NAME, None):
        raise SeaplaneError(
            f"OPENAI API Key `{OPENAI_API_KEY_NAME}` is not set,\
                  use `sea.config.set_api_keys`."
        )


def _gpt_3() -> Callable[[Dict[str, Any]], Any]:
    def model(payload: Dict[str, Any]) -> Any:
        """
        The OpenAI GPT-3 payload.

        payload = {"model": "text-davinci-003", "prompt": prompt, "temperature": 0}
        """
        _check_openai_api_keys()

        url = f"{OPENAI_URL}/v1/completions"

        return unwrap(
            handle_request(
                lambda: requests.post(
                    url,
                    json=payload,
                    headers=headers(config._api_keys[OPENAI_API_KEY_NAME]),
                )
            )
        )

    return model


def _gpt_3_5() -> Callable[[Dict[str, Any]], Any]:
    def model(payload: Dict[str, Any]) -> Any:
        """
        The OpenAI GPT-3.5 payload.

         payload = {
           "model": "gpt-3.5-turbo",
           "messages": [{"role": "user", "content": "Any prompt here.."}],
           "temperature": 0.7,
         }
        """
        _check_openai_api_keys()

        url = f"{OPENAI_URL}/v1/chat/completions"

        return unwrap(
            handle_request(
                lambda: requests.post(
                    url,
                    json=payload,
                    headers=headers(config._api_keys[OPENAI_API_KEY_NAME]),
                )
            )
        )

    return model


class OpenAI:
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

        if self.model == "gpt-3.5":
            log.info("Processing GPT-3.5 Model...")
            self.args = self.args + (_gpt_3_5(),)

            return self.func(*self.args, **self.kwargs)
        elif self.model == "gpt-3":
            log.info("Processing GPT-3 Model...")
            self.args = self.args + (_gpt_3(),)

            return self.func(*self.args, **self.kwargs)
        else:
            raise SeaplaneError(
                "OPENAI Task Not recognised , the model \
                    doesn't match with any current OPEN AI task."
            )

    def print(self) -> None:
        log.info(f"id: {self.id}, type: {self.type}, model: {self.model}")
