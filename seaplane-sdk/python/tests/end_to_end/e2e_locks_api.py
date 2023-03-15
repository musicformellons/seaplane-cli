"""
End to End tests to Locks Coordination API.

This tests can't be ran more than once every 1 minute.
"""

from typing import List, Tuple

from seaplane import sea
from seaplane.model import LockPage, Name

from . import E2E_API_KEY

ACQUIRE_LOCK_NAME = Name(name=b"foo/bar/acquire")
RENEW_LOCK_NAME = Name(name=b"foo/bar/renew")


def test_get_page_should_be_empty() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    assert sea.locks.get_page() == LockPage(locks=[], next_lock=None)


def test_acquire() -> None:
    sea.config.set_api_key(E2E_API_KEY)
    acquire_result = sea.locks.acquire(ACQUIRE_LOCK_NAME, "test_acquire", 2)

    sea.locks.release(ACQUIRE_LOCK_NAME, acquire_result.id)
    assert acquire_result.sequencer > 1


def test_release_lock() -> None:
    sea.config.set_api_key(E2E_API_KEY)
    value = sea.locks.acquire(ACQUIRE_LOCK_NAME, "test_release_lock", 60)

    assert value.sequencer > 0
    assert sea.locks.release(ACQUIRE_LOCK_NAME, value.id)
    assert sea.locks.get_page().locks == []


def test_get_page_should_returns_one_element_when_acquire() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    heldlock = sea.locks.acquire(ACQUIRE_LOCK_NAME, "test_acquire", 10)

    lock_page = sea.locks.get_page()

    sea.locks.release(ACQUIRE_LOCK_NAME, heldlock.id)
    assert len(lock_page.locks) == 1


def test_renew_lock() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    heldlock = sea.locks.acquire(RENEW_LOCK_NAME, "test_renew_lock", 10)

    assert sea.locks.renew(RENEW_LOCK_NAME, heldlock.id, 15)
    assert sea.locks.get_all_pages()[0].info.ttl == 15

    sea.locks.release(RENEW_LOCK_NAME, heldlock.id)


def test_get_all_pages_should_return_25_elements_using_pagination() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    created_locks: List[Tuple[Name, str]] = []

    for i in range(0, 25):
        lock_name = Name(f"foo/bar/acquire/{i}".encode())
        heldlock = sea.locks.acquire(lock_name, "test_get_all_pages_" + str(i), 60)
        created_locks.append((lock_name, heldlock.id))

    locks = sea.locks.get_all_pages()

    for lock in created_locks:
        sea.locks.release(lock[0], lock[1])

    assert len(locks) == 25
