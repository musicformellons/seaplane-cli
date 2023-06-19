from typing import Any, Callable

import requests
from requests import Response
from returns.result import Failure, Result, Success

from ..logging import log
from ..model.errors import HTTPError
from .api_http import SDK_HTTP_ERROR_CODE
from .token_api import TokenAPI


def handle_request(request: Callable[[], Response]) -> Result[Any, HTTPError]:
    try:
        response = request()

        if response.ok:
            return Success(response.json())
        else:
            body_error = response.text
            log.error(f"Request Error: {body_error}")
            return Failure(HTTPError(response.status_code, body_error))
    except requests.exceptions.RequestException as err:
        log.error(f"Request exception: {str(err)}")
        return Failure(HTTPError(SDK_HTTP_ERROR_CODE, str(err)))


def provision_req(
    token_api: TokenAPI,
) -> Callable[[Callable[[str], Response]], Result[Any, HTTPError]]:
    """
    Before every request, we make sure we use a valid access token.
    """

    def handle_request(request: Callable[[str], Response], token: str) -> Result[Any, HTTPError]:
        try:
            response = request(token)

            if response.ok:
                data = None
                if response.headers.get("content-type") == "application/json":
                    data = response.json()
                else:
                    data = response.text

                return Success(data)
            else:
                body_error = response.text
                log.error(f"Request Error: {body_error}")
                return Failure(HTTPError(response.status_code, body_error))
        except requests.exceptions.RequestException as err:
            log.error(f"Request exception: {str(err)}")
            return Failure(HTTPError(SDK_HTTP_ERROR_CODE, str(err)))

    def renew_if_fails(
        token_api: TokenAPI, request: Callable[[str], Response], http_error: HTTPError
    ) -> Result[Any, HTTPError]:
        if http_error.status != 401:
            return Failure(http_error)

        if token_api.auto_renew:
            log.info("Auto-Renew, renewing the token...")
            token = token_api.renew_token()
            return handle_request(request, token)
        else:
            return Failure(http_error)

    def req(request: Callable[[str], Response]) -> Result[Any, HTTPError]:
        access_token: Result[str, HTTPError]

        if token_api.access_token is not None:
            access_token = Success(token_api.access_token)
        else:
            access_token = token_api._request_access_token().map(lambda result: result["token"])

        return access_token.bind(lambda token: handle_request(request, token)).lash(
            lambda error: renew_if_fails(token_api, request, error)
        )

    return req
