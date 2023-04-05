from typing import Any, Generator

import pytest
import requests_mock

from seaplane import sea
from seaplane.api.lock_api import LockAPI
from seaplane.configuration import Configuration
from seaplane.model import HeldLock, Lock, LockInfo, LockPage, Name
from seaplane.model.errors import HTTPError

from ..conftest import add_token_request


@pytest.fixture
def locks_get_page_root_directory() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/locks",
            additional_matcher=match_authorization,
            status_code=200,
            json={
                "locks": [
                    {
                        "name": "bG9jay10ZXN0",
                        "id": "BiqhSv0tuAk",
                        "info": {"ttl": 1000, "client-id": "test", "ip": ""},
                    }
                ],
                "next": None,
            },
        )

        yield


@pytest.fixture
def fails_get_page_root_directory() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/locks",
            status_code=400,
            text="Some error",
        )

        yield


@pytest.fixture
def locks_get_page_another_directory() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9v/",
            additional_matcher=match_authorization,
            status_code=200,
            json={
                "locks": [
                    {
                        "name": "Zm9vL2Jhcg",
                        "id": "BiqhSv0tuAk",
                        "info": {"ttl": 1000, "client-id": "test", "ip": ""},
                    }
                ],
                "next": None,
            },
        )

        yield


@pytest.fixture
def get_lock() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json={
                "name": "Zm9vL2Jhcg",
                "id": "BiqhSv0tuAk",
                "info": {"ttl": 1000, "client-id": "test", "ip": ""},
            },
        )

        yield


@pytest.fixture
def acquire_lock() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return (
                request.headers["Authorization"] == "Bearer This is a token"  # noqa
                and request.query == "client-id=client-id&ttl=60"
            )

        requests_mocker.post(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json={"id": "AOEHFRa4Ayg", "sequencer": 3},
        )

        yield


@pytest.fixture
def release_lock() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return (
                request.headers["Authorization"] == "Bearer This is a token"  # noqa
                and request.query == "id=aoehfra4ayg"
            )

        requests_mocker.delete(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json="OK",
        )

        yield


@pytest.fixture
def fails_release_lock() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        requests_mocker.delete(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9vL2Jhcg",
            status_code=400,
            text="Some error",
        )

        yield


@pytest.fixture
def renew_lock() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return (
                request.headers["Authorization"] == "Bearer This is a token"  # noqa
                and request.query == "id=aoehfra4ayg&ttl=60"
            )

        requests_mocker.patch(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json="OK",
        )

        yield


@pytest.fixture
def lock_api() -> Generator[LockAPI, None, None]:
    configuration = Configuration()
    configuration.set_api_key("api_key")
    lock_api = LockAPI(configuration)

    yield lock_api


def test_locks_get_page_of_root_directory(  # type: ignore
    lock_api, locks_get_page_root_directory
) -> None:
    assert lock_api.get_page() == LockPage(
        locks=[
            Lock(
                id="BiqhSv0tuAk",
                name=Name(name=b"lock-test"),
                info=LockInfo(ttl=1000, client_id="test", ip=""),
            )
        ],
        next_lock=None,
    )


def test_locks_get_page_of_root_directory_should_raise_http_error_400(  # type: ignore
    lock_api, fails_get_page_root_directory
) -> None:
    try:
        lock_api.get_page()
        pytest.fail("get_page must throw an HTTPError")
    except HTTPError as http_error:
        assert http_error.status == 400
        assert http_error.message == "Some error"


def test_locks_get_page_of_another_directory(  # type: ignore
    lock_api, locks_get_page_another_directory
) -> None:
    assert lock_api.get_page(directory=Name(b"foo")) == LockPage(
        locks=[
            Lock(
                id="BiqhSv0tuAk",
                name=Name(b"foo/bar"),
                info=LockInfo(ttl=1000, client_id="test", ip=""),
            )
        ],
        next_lock=None,
    )


def test_get_lock(lock_api, get_lock) -> None:  # type: ignore
    assert lock_api.get(Name(b"foo/bar")) == Lock(
        id="BiqhSv0tuAk",
        name=Name(b"foo/bar"),
        info=LockInfo(ttl=1000, client_id="test", ip=""),
    )


def test_acquire_lock(lock_api, acquire_lock) -> None:  # type: ignore
    assert lock_api.acquire(Name(b"foo/bar"), "client-id", 60) == HeldLock(
        id="AOEHFRa4Ayg", sequencer=3
    )


def test_release_lock(lock_api, release_lock) -> None:  # type: ignore
    assert lock_api.release(Name(b"foo/bar"), "AOEHFRa4Ayg")


def test_release_lock_should_raise_http_error_400(  # type: ignore
    lock_api, fails_release_lock
) -> None:
    try:
        lock_api.release(Name(b"foo/bar"), "AOEHFRa4Ayg")
        pytest.fail("get_page must throw an HTTPError")
    except HTTPError as http_error:
        assert http_error.status == 400
        assert http_error.message == "Some error"


def test_renew_lock(lock_api, renew_lock) -> None:  # type: ignore
    assert lock_api.renew(Name(b"foo/bar"), "AOEHFRa4Ayg", 60)


def test_locks_get_page_using_default_instance(  # type: ignore
    locks_get_page_root_directory,
) -> None:
    sea.config.set_api_key("api_key")

    assert sea.locks.get_page() == LockPage(
        locks=[
            Lock(
                id="BiqhSv0tuAk",
                name=Name(b"lock-test"),
                info=LockInfo(ttl=1000, client_id="test", ip=""),
            )
        ],
        next_lock=None,
    )
