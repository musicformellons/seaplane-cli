from typing import Generator

import pytest

from seaplane.api.token_api import TokenAPI
from seaplane.configuration import Configuration
from seaplane.model.errors import HTTPError


@pytest.fixture
def token_api() -> Generator[TokenAPI, None, None]:
    config = Configuration()
    config.set_api_key("api_key")
    token_api = TokenAPI(config)

    yield token_api


def test_given_token_post_call_returns_the_access_token(  # type: ignore
    token_api, success_token_post
) -> None:
    assert token_api.get_token() == "This is a token"


def test_given_token_post_failure_call_returns_the_error_code(  # type: ignore
    token_api, fail_token_post
) -> None:
    try:
        token_api.get_token()
    except HTTPError as http_error:
        assert http_error.status == 400
        assert http_error.message == ""


def test_given_token_post_call_save_the_token_locally(  # type: ignore
    token_api, success_token_post
) -> None:
    token_api.get_token()
    assert token_api.access_token == "This is a token"


def test_given_token_api_auto_renew_should_be_true(token_api) -> None:  # type: ignore
    assert token_api.auto_renew is True


def test_given_set_token_should_set_auto_renew_to_false(token_api) -> None:  # type: ignore
    token_api.set_token("This is a token")
    assert token_api.auto_renew is False


def test_given_set_token_should_set_access_token(token_api) -> None:  # type: ignore
    token_api.set_token("This is a token")
    assert token_api.access_token == "This is a token"


def test_given_renew_token_should_set_a_new_access_token(  # type: ignore
    token_api, success_token_post
) -> None:
    token_api.set_token("This is an old token")
    token_api.renew_token()
    assert token_api.access_token == "This is a token"
