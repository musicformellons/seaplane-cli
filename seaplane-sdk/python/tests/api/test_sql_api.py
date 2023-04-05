from typing import Any, Generator

import pytest
import requests_mock

from seaplane.api.sql_api import GlobalSQL
from seaplane.configuration import Configuration
from seaplane.model import CreatedDatabase

from ..conftest import add_token_request


@pytest.fixture
def sql_api() -> Generator[GlobalSQL, None, None]:
    configuration = Configuration()
    configuration.set_api_key("api_key")
    sql_api = GlobalSQL(configuration)

    yield sql_api


@pytest.fixture
def create_database() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return (
                request.headers["Authorization"] == "Bearer This is a token"  # noqa
                and request.text == "{}"
            )

        requests_mocker.post(
            "https://sql.cplane.cloud/v1/databases",
            additional_matcher=match_authorization,
            status_code=200,
            json={
                "database": "graceful-jewel",
                "username": "cute-dress",
                "password": "_password_",
            },
        )

        yield


def test_create_database_should_returns_a_created_database(  # type: ignore
    sql_api, create_database
) -> None:
    assert sql_api.create_database() == CreatedDatabase(
        name="graceful-jewel",
        username="cute-dress",
        password="_password_",
    )
