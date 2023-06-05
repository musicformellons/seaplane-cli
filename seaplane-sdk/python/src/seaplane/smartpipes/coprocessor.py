from typing import Any, Callable, Dict, Optional, Tuple

import requests

from ..api.api_http import headers
from ..api.api_request import handle_request, provision_req
from ..configuration import config
from ..model.errors import SeaplaneError
from ..util import unwrap


class CoprocessorEvent:
    def __init__(self, id: str, input: Any) -> None:
        self.id = id
        self.status = "in_progress"
        self.input = input
        self.output: Optional[Any] = None
        self.error: Optional[Any] = None

    def set_ouput(self, output: Any) -> None:
        self.output = output
        self.status = "completed"

    def set_error(self, error: Any) -> None:
        self.error = error
        self.status = "error"


SEAPLANE_API_KEY_NAME = "SEA_API_KEY"
OPENAI_API_KEY_NAME = "OPENAI_API_KEY"
REPLICATE_API_KEY_NAME = "RE_API_KEY"


def inference(version: str) -> Callable[[Dict[str, Any]], Any]:
    def model(input_data: Dict[str, Any]) -> Any:
        if config._api_keys is None:
            raise SeaplaneError(
                "Replicate API Key `RE_API_KEY` is not set, use `sea.config.set_api_keys`."
            )
        elif not config._api_keys.get(REPLICATE_API_KEY_NAME, None):
            raise SeaplaneError(
                "Replicate API Key `RE_API_KEY` is not set, use `sea.config.set_api_keys`."
            )

        headers = {"Authorization": f"Token {config._api_keys[REPLICATE_API_KEY_NAME]}"}

        payload = {
            "version": version,
            "input": {
                "width": 512,
                "height": 512,
                "num_outputs": 4,
                "prompt": input_data,
                "seed": 11,
            },
        }
        response = requests.post(
            "https://api.replicate.com/v1/predictions",
            json=payload,
            headers=headers,
        )

        if response.ok:
            result = response.json()
            id = result["id"]

            while not result.get("output", None):
                endpoint = f"https://api.replicate.com/v1/predictions/{id}"
                response = requests.get(
                    endpoint,
                    headers=headers,
                )

                print("Loading...")
                if response.ok:
                    result = response.json()

                    if result.get("output", None) is not None:
                        return result
        else:
            return response.text

    return model


def bloom() -> Callable[[Dict[str, Any]], Any]:
    def model(input: Dict[str, Any]) -> Any:
        url = "https://substation.dev.cplane.cloud/v1/completions"
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


def gpt_3() -> Callable[[str], Any]:
    def model(prompt: str) -> Any:
        if config._api_keys is None:
            raise SeaplaneError(
                f"OPENAI API Key `{OPENAI_API_KEY_NAME}` is not set, use `sea.config.set_api_keys`."  # noqa
            )
        elif not config._api_keys.get(OPENAI_API_KEY_NAME, None):
            raise SeaplaneError(
                f"OPENAI API Key `{OPENAI_API_KEY_NAME}` is not set, use `sea.config.set_api_keys`."  # noqa
            )

        url = "https://api.openai.com/v1/completions"

        payload = {"model": "text-davinci-003", "prompt": prompt, "temperature": 0}

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


def gpt_3_5() -> Callable[[str], Any]:
    def model(input: str) -> Any:
        if config._api_keys is None:
            raise SeaplaneError(
                f"OPENAI API Key `{OPENAI_API_KEY_NAME}` is not set, use `sea.config.set_api_keys`."  # noqa
            )
        elif not config._api_keys.get(OPENAI_API_KEY_NAME, None):
            raise SeaplaneError(
                f"OPENAI API Key `{OPENAI_API_KEY_NAME}` is not set, use `sea.config.set_api_keys`."  # noqa
            )

        url = "https://api.openai.com/v1/chat/completions"

        payload = {
            "model": "gpt-3.5-turbo",
            "messages": [{"role": "user", "content": input}],
            "temperature": 0.7,
        }

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


class Coprocessor:
    def __init__(self, func: Callable[[Any], Any], type: str, id: str, model: Optional[str]) -> None:
        self.func = func
        self.args: Optional[Tuple[Any, ...]] = None
        self.kwargs: Optional[Dict[str, Any]] = None
        self.type = type
        self.model = model
        self.id = id

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        print(f"Coprocessor type '{self.type}' Model ID {self.model}")

        if self.model == "bloom":
            print("Processing Bloom Model...")
            self.args = self.args + (bloom(),)

            return self.func(*self.args, **self.kwargs)
        elif self.model == "gpt-3.5":
            print("Procesing GPT-3.5 Model...")
            self.args = self.args + (gpt_3_5(),)

            return self.func(*self.args, **self.kwargs)
        elif self.model == "gpt-3":
            print("Procesing GPT-3 Model...")
            self.args = self.args + (gpt_3(),)

            return self.func(*self.args, **self.kwargs)
        elif self.model == "stable-diffusion":
            print("Accessing Replicate coprocessor...")
            self.args = self.args + (
                inference("27b93a2413e7f36cd83da926f3656280b2931564ff050bf9575f1fdf9bcd7478"),
            )

            return self.func(*self.args, **self.kwargs)
        elif self.model:
            print("Accessing Replicate coprocessor...")
            self.args = self.args + (inference(self.model),)

            return self.func(*self.args, **self.kwargs)
        else:
            return self.func(*self.args, **self.kwargs)

    def print(self) -> None:
        print(f"id: {self.id}, type: {self.type}, model: {self.model}")
