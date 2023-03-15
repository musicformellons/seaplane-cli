from typing import Generator

import pytest

from seaplane.api.restrict_api import RestrictAPI
from seaplane.configuration import Configuration
from seaplane.model import Key
from seaplane.model.errors import SeaplaneError


@pytest.fixture
def restrict_api() -> Generator[RestrictAPI, None, None]:
    configuration = Configuration()
    configuration.set_api_key("api_key")
    restrict_api = RestrictAPI(configuration)

    yield restrict_api


def test_get_all_page_should_raise_error_when_api_is_none_with_from_restriction(  # type: ignore
    restrict_api,
) -> None:
    try:
        restrict_api.get_all_pages(api=None, from_restriction=Key(b"foo/bar"))
        pytest.fail("get_all_page must throw an SeaplaneError")
    except SeaplaneError as sea_error:
        assert str(sea_error) == "You must set api with from_restriction parameters."
