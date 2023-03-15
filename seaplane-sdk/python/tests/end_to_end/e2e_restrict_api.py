"""
End to End tests to Restrict Coordination API.
"""

from typing import List, Tuple
import uuid

from seaplane import sea
from seaplane.model import (
    Key,
    KeyValue,
    Name,
    Provider,
    Region,
    Restriction,
    RestrictionDetails,
    RestrictionState,
    SeaplaneApi,
)

from . import E2E_API_KEY


def test_before_all() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    sea.locks.acquire(Name(b"foo/bar"), "test_acquire", 10)
    sea.metadata.set(KeyValue(b"key", b"value"))


def test_set_lock_restriction() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    restriction_details = RestrictionDetails(
        providers_allowed=[Provider.aws],
        providers_denied=[Provider.azure],
        regions_allowed=[Region.europe],
        regions_denied=[Region.oceania],
    )

    assert sea.restrict.set(SeaplaneApi.locks, Key(b"foo/bar"), restriction_details) is True


def test_set_metadata_restriction() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    restriction_details = RestrictionDetails(
        providers_allowed=[Provider.azure],
        providers_denied=[Provider.aws],
        regions_allowed=[Region.oceania],
        regions_denied=[Region.europe],
    )

    assert sea.restrict.set(SeaplaneApi.metadata, Key(b"key"), restriction_details) is True


def test_get_lock_restriction() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    assert sea.restrict.get(SeaplaneApi.locks, Key(b"foo/bar")) == Restriction(
        SeaplaneApi.locks,
        Key(b"foo/bar"),
        RestrictionDetails(
            regions_allowed=[Region.europe],
            regions_denied=[Region.oceania],
            providers_allowed=[Provider.aws],
            providers_denied=[Provider.azure],
        ),
        state=RestrictionState.pending,
    )


def test_get_metadata_restriction() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    assert sea.restrict.get(SeaplaneApi.metadata, Key(b"key")) == Restriction(
        SeaplaneApi.metadata,
        Key(b"key"),
        RestrictionDetails(
            regions_allowed=[Region.oceania],
            regions_denied=[Region.europe],
            providers_allowed=[Provider.azure],
            providers_denied=[Provider.aws],
        ),
        state=RestrictionState.pending,
    )


def test_get_page_should_returns_locks_and_metadata_restrictions() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    assert len(sea.restrict.get_page().restrictions) == 2


def test_get_page_should_returns_locks_only_restriction() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    page = sea.restrict.get_page(SeaplaneApi.locks)

    assert len(page.restrictions) == 1
    assert page.restrictions[0].api == SeaplaneApi.locks


def test_get_page_should_returns_metadata_only_restriction() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    page = sea.restrict.get_page(SeaplaneApi.metadata)

    assert len(page.restrictions) == 1
    assert page.restrictions[0].api == SeaplaneApi.metadata


def test_get_all_pages_should_return_26_elements_using_pagination() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    created_locks: List[Tuple[Name, str]] = []
    created_metadata: List[Key] = []

    for i in range(0, 12):
        lock_name = Name(f"foo/bar/acquire/{uuid.uuid4()}".encode())
        heldlock = sea.locks.acquire(lock_name, "test_restrict_get_all_pages_" + str(i), 60)
        created_locks.append((lock_name, heldlock.id))

        restriction_detail = RestrictionDetails(
            regions_allowed=[Region.oceania],
            regions_denied=[Region.europe],
            providers_allowed=[Provider.azure],
            providers_denied=[Provider.aws],
        )
        sea.restrict.set(SeaplaneApi.locks, Key(lock_name.name), restriction_detail)

    for i in range(0, 12):
        key = Key(f"foo/bar/{uuid.uuid4()}".encode())
        success = sea.metadata.set(KeyValue(key.key, ("value " + str(i)).encode()))
        if success:
            created_metadata.append(key)

            restriction_detail = RestrictionDetails(
                regions_allowed=[Region.oceania],
                regions_denied=[Region.europe],
                providers_allowed=[Provider.azure],
                providers_denied=[Provider.aws],
            )
            sea.restrict.set(SeaplaneApi.metadata, key, restriction_detail)

    restrictions = sea.restrict.get_all_pages()

    for lock in created_locks:
        sea.locks.release(lock[0], lock[1])

    for metadata in created_metadata:
        sea.metadata.delete(metadata)

    for restriction in restrictions:
        sea.restrict.delete(restriction.api, restriction.directory)

    assert len(restrictions) == 26
    assert len(sea.restrict.get_all_pages()) == 0
