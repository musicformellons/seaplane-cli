from typing import Any, Generator

import pytest
import requests_mock

from seaplane import sea
from seaplane.api.metadata_api import MetadataAPI
from seaplane.configuration import Configuration
from seaplane.model import KeyString, KeyValue, KeyValuePage, KeyValueStream, KeyValueString

from ..conftest import add_token_request
from ..util import get_absolute_path, get_file_bytes


@pytest.fixture
def get_contents_of_root_directory() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/config",
            additional_matcher=match_authorization,
            status_code=200,
            json={"kvs": [{"key": "Zm9vL2Jhcgo", "value": "Ynll"}], "next_key": None},
        )

        yield


@pytest.fixture
def get_page_of_directory() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/config/base64:Zm9v/",
            additional_matcher=match_authorization,
            status_code=200,
            json={"kvs": [{"key": "Zm9vL2Jhcgo", "value": "Ynll"}], "next_key": "Zm9vL2Zvbw"},
        )

        yield


@pytest.fixture
def get_key_value_pair_decoding_in_base64url() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.get(
            "https://metadata.cplane.cloud/v1/config/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json={"key": "Zm9vL2Jhcg", "value": "Ynll"},
        )

        yield


@pytest.fixture
def delete_key_value_pair_decoding_in_base64url() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"  # noqa

        requests_mocker.delete(
            "https://metadata.cplane.cloud/v1/config/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json="Ok",
        )

        yield


@pytest.fixture
def set_key_value_pair() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return (
                request.headers["Authorization"] == "Bearer This is a token"  # noqa
                and request.text == "ZW1wdHk"
            )

        requests_mocker.put(
            "https://metadata.cplane.cloud/v1/config/base64:YmFyL2Zvbw",
            additional_matcher=match_authorization,
            status_code=200,
            json="Ok",
        )

        yield


@pytest.fixture
def set_key_binary_value_pair() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            lena_base64 = get_file_bytes(
                relative_path="fixtures/api/seaplane_img_in_base64.txt"
            ).decode("utf-8")

            return (
                request.headers["Authorization"] == "Bearer This is a token"  # noqa
                and request.text == lena_base64
            )

        requests_mocker.put(
            "https://metadata.cplane.cloud/v1/config/base64:YmFyL2Zvbw",
            additional_matcher=match_authorization,
            status_code=200,
            json="Ok",
        )

        yield


@pytest.fixture
def metadata_api() -> Generator[MetadataAPI, None, None]:
    configuration = Configuration()
    configuration.set_api_key("api_key")
    metadata_api = MetadataAPI(configuration)

    yield metadata_api


def test_given_metadata_get_contents_of_root_directory(  # type: ignore
    metadata_api, get_contents_of_root_directory
) -> None:
    assert metadata_api.get_page() == KeyValuePage(
        key_value_pairs=[KeyValue(key="foo/bar\n".encode(), value="bye".encode())],
        next_key=None,
    )


def test_given_metadata_get_a_key_value_pair(  # type: ignore
    metadata_api, get_key_value_pair_decoding_in_base64url
) -> None:
    assert metadata_api.get(KeyString("foo/bar")) == KeyValue(
        key="foo/bar".encode(), value="bye".encode()
    )


def test_given_metadata_delete_a_key_value_pair(  # type: ignore
    metadata_api, delete_key_value_pair_decoding_in_base64url
) -> None:
    assert metadata_api.delete(KeyString("foo/bar"))


def test_given_metadata_set_key_value_pair(  # type: ignore
    metadata_api, set_key_value_pair
) -> None:
    assert metadata_api.set(KeyValueString("bar/foo", "empty"))


def test_given_metadata_set_key_binary_value_pair(  # type: ignore
    metadata_api, set_key_binary_value_pair
) -> None:
    file_path = get_absolute_path("fixtures/metadata/seaplane.jpeg")
    assert metadata_api.set(KeyValueStream(b"bar/foo", open(file_path, "rb")))


def test_given_metadata_using_default_instance(  # type: ignore
    get_contents_of_root_directory,
) -> None:
    sea.config.set_api_key("api_key")

    assert sea.metadata.get_page() == KeyValuePage(
        key_value_pairs=[KeyValue(key="foo/bar\n".encode(), value="bye".encode())],
        next_key=None,
    )


def test_given_metadata_get_page_of_directory(  # type: ignore
    metadata_api, get_page_of_directory
) -> None:
    assert metadata_api.get_page(directory=KeyString("foo")) == KeyValuePage(
        key_value_pairs=[KeyValue(key="foo/bar\n".encode(), value="bye".encode())],
        next_key=KeyString("foo/foo"),
    )
