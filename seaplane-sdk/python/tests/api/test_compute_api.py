from typing import Any, Generator

import pytest
import requests_mock

from seaplane import sea
from seaplane.api.compute_api import ComputeAPI
from seaplane.configuration import Configuration
from seaplane.model import Flight, Formation, FormationPage, MetaPage

from ..conftest import add_token_request


@pytest.fixture
def get_formations_page() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"

        requests_mocker.get(
            "https://compute.cplane.cloud/v2beta/formations",
            additional_matcher=match_authorization,
            status_code=200,
            json={
                "objects": [
                    {
                        "oid": "frm-0oug6ng05tvll000e14k2sd3og",
                        "name": "example-formation",
                        "url": "https://example-formation.tenant.on.cplane.cloud/",
                        "flights": [
                            {
                                "name": "example-flight",
                                "oid": "flt-0oug6ng05tvll000e14k2sd3og",
                                "image": "registry.cplane.clou"
                                + "d/seaplane-demo/nginxdemos/hello:latest",
                                "status": "healthy",
                            }
                        ],
                        "gateway-flight": "example-flight",
                    }
                ],
                "meta": {
                    "total": 1,
                    "next": "http://example-next.com",
                    "prev": "http://example-prev.com",
                },
            },
        )

        yield


@pytest.fixture
def get_formation() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"

        requests_mocker.get(
            "https://compute.cplane.cloud/v2beta/formations/frm-0oug6ng05tvll000e14k2sd3og",
            additional_matcher=match_authorization,
            status_code=200,
            json={
                "oid": "frm-0oug6ng05tvll000e14k2sd3og",
                "name": "example-formation",
                "url": "https://example-formation.tenant.on.cplane.cloud/",
                "flights": [
                    {
                        "name": "example-flight",
                        "oid": "flt-0oug6ng05tvll000e14k2sd3og",
                        "image": "registry.cplane.cloud/seaplane-demo/nginxdemos/hello:latest",
                        "status": "healthy",
                    }
                ],
                "gateway-flight": "example-flight",
            },
        )

        yield


@pytest.fixture
def delete_formation() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"

        requests_mocker.delete(
            "https://compute.cplane.cloud/v2beta/formations/frm-0oug6ng05tvll000e14k2sd3og",
            additional_matcher=match_authorization,
            status_code=200,
            json="Ok",
        )

        yield


@pytest.fixture
def create_formation() -> Generator[None, None, None]:
    with requests_mock.Mocker() as requests_mocker:
        add_token_request(requests_mocker)

        def match_authorization(request: Any) -> Any:
            return request.headers["Authorization"] == "Bearer This is a token"

        requests_mocker.post(
            "https://metadata.cplane.cloud/v1/locks/base64:Zm9vL2Jhcg",
            additional_matcher=match_authorization,
            status_code=200,
            json={"id": "AOEHFRa4Ayg", "sequencer": 3},
        )

        yield


DEFAULT_FORMATION = Formation(
    oid="frm-0oug6ng05tvll000e14k2sd3og",
    name="example-formation",
    url="https://example-formation.tenant.on.cplane.cloud/",
    flights=[
        Flight(
            name="example-flight",
            oid="flt-0oug6ng05tvll000e14k2sd3og",
            image="registry.cplane.cloud/seaplane-demo/nginxdemos/hello:latest",
            status="healthy",
        )
    ],
    gateway_flight="example-flight",
)


@pytest.fixture
def compute_api() -> Generator[ComputeAPI, None, None]:
    configuration = Configuration()
    configuration.set_api_key("api_key")
    metadata_api = ComputeAPI(configuration)

    yield metadata_api


def test_given_compute_using_default_instance(  # type: ignore
    get_formations_page,
) -> None:
    sea.config.set_api_key("api_key")

    assert sea.compute.get_page() == FormationPage(
        formations=[DEFAULT_FORMATION],
        meta=MetaPage(total=1, next="http://example-next.com", prev="http://example-prev.com"),
    )


def test_given_compute_get_formations_page_returns_formations_and_meta(  # type: ignore
    compute_api,
    get_formations_page,
) -> None:

    assert compute_api.get_page() == FormationPage(
        formations=[DEFAULT_FORMATION],
        meta=MetaPage(total=1, next="http://example-next.com", prev="http://example-prev.com"),
    )


def test_given_compute_get_formation_returns_a_formation(  # type: ignore
    compute_api,
    get_formation,
) -> None:

    assert compute_api.get("frm-0oug6ng05tvll000e14k2sd3og") == DEFAULT_FORMATION


def test_given_compute_delete_formation_returns_ok(  # type: ignore
    compute_api,
    delete_formation,
) -> None:

    assert compute_api.delete("frm-0oug6ng05tvll000e14k2sd3og") is None
