from typing import Any, Dict, List, NamedTuple, Optional

from ...util.base64url import base64url_decode_to_bytes


class Name(NamedTuple):
    """
    bytes Name of a lock.
    """

    name: bytes


class NameString(Name):
    """
    String Name of a lock.

    Attributes
        ----------
        name : bytes
            name of a lock in bytes.
        name_str: str
            value of a lock in string
    """

    name_str: str

    def __new__(cls, name: str, encoding: str = "utf-8", **kwargs):  # type: ignore
        return super().__new__(cls, name.encode(encoding), **kwargs)

    def __init__(self, name: str):
        self.name_str = name


class LockInfo(NamedTuple):
    """
    Lock information class.

    Attributes
        ----------
        ttl : int
            This is the requested time to live, in seconds.
        client_id: str
            Client-chosen identifier stored with the lock for informational purposes.
        ip: str
            Lock IP

    """

    ttl: int
    client_id: str
    ip: str


class Lock(NamedTuple):
    """
    Lock class.

    Attributes
        ----------
        id : str
            Lock ID.
        name: Name
            Lock name.
        info: LockInfo
            Additional information of a lock.
    """

    id: str
    name: Name
    info: LockInfo


class LockPage(NamedTuple):
    """
    Lock Page class.

    It contains a paginated list of locks, next_lock is used for the next call.

    Attributes
        ----------
        locks : List[Lock]
            list of locks.
        next_lock: Optional[Name]
            If next_lock is non-null and you want to get the next LockPage,
            you can repeat a query using next_lock to continue getting LockPage.
    """

    locks: List[Lock]
    next_lock: Optional[Name]


class HeldLock(NamedTuple):
    """
    HeldLock class.

    Attributes
        ----------
        id : str
            The ID of the lock, to be used in release and renew calls.
        sequencer: int
            A sequencer representing the number of times the lock has gone
            from held to free, can be used to safely coordinate
            access with external resources.
    """

    id: str
    sequencer: int


def to_lock_page(lock_range: Dict[str, Any]) -> LockPage:
    return LockPage(
        locks=[to_lock(lock) for lock in lock_range["locks"]],
        next_lock=_to_name(lock_range["next"]),
    )


def _to_name(name: Optional[str]) -> Optional[Name]:
    if name is None:
        return None

    return Name(base64url_decode_to_bytes(name))


def to_lock(lock: Dict[str, Any]) -> Lock:
    lock_name = _to_name(lock["name"])
    lock_name = Name(b"") if lock_name is None else lock_name
    return Lock(id=lock["id"], name=lock_name, info=to_lock_info(lock["info"]))


def to_lock_info(lock_info: Dict[str, Any]) -> LockInfo:
    return LockInfo(ttl=lock_info["ttl"], client_id=lock_info["client-id"], ip=lock_info["ip"])


def to_held_lock(lock: Dict[str, Any]) -> HeldLock:
    return HeldLock(id=lock["id"], sequencer=lock["sequencer"])
