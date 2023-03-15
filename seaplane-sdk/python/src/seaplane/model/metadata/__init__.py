from typing import Any, BinaryIO, Dict, List, NamedTuple, Optional

from ...util.base64url import base64url_decode_to_bytes


class Key(NamedTuple):
    """
    bytes Key of the key-value pair.
    """

    key: bytes


class KeyString(Key):
    """
    String Key of the key-value pair.

    Attributes
        ----------
        key : bytes
            key of the key-value pair
        key_str: str
            key of the key-value pair in string
    """

    key_str: str

    def __new__(cls, key: str, encoding: str = "utf-8", **kwargs):  # type: ignore
        return super().__new__(cls, key.encode(encoding), **kwargs)

    def __init__(self, key: str):
        self.key_str = key


class KeyValue(NamedTuple):
    """
    bytes Key Value class.

    Attributes
        ----------
        key : bytes
            key of the key-value pair.
        value: bytes
            value of the key-value pair.
    """

    key: bytes
    value: bytes


class KeyValueString(KeyValue):
    """
    String Key Value class.

    Attributes
        ----------
        key : bytes
            key of the key-value pair.
        value: bytes
            value of the key-value pair.
        key_str : str
            key of the key-value pair in String
        value_str: str
            value of the key-value pair in String
    """

    key_str: str
    value_str: str

    def __new__(cls, key, value, encoding="utf-8", **kwargs):  # type: ignore
        return super().__new__(cls, key.encode(encoding), value.encode(encoding), **kwargs)

    def __init__(self, key: str, value: str):
        self.key_str = key
        self.value_str = value


class KeyValueStream(KeyValue):
    """
    bytes Key and IO stream Value class.

    Attributes
        ----------
        key : bytes
            key of the key-value pair.
        value: BinaryIO
            value of the key-value.
    """

    value_stream: BinaryIO

    def __new__(cls, key, value, **kwargs):  # type: ignore
        content: bytes = b""

        with value as binary_stream:
            content = binary_stream.read()

        return super().__new__(cls, key, content, **kwargs)

    def __init__(self, key: bytes, value: BinaryIO):
        self.value_stream = value


class KeyValuePage(NamedTuple):
    """
    bytes Key Value Page class.

    It contains a paginated list of key-value, next_key is used for the next call.

    Attributes
        ----------
        key_value_pairs : KeyValue
            list of key-value
        next_key: Optional[Key]
            If next_key is non-null and you want to get the next KeyValuePage,
            you can repeat a query using next_key to continue getting KeyValuePage.
    """

    key_value_pairs: List[KeyValue]
    next_key: Optional[Key]


def to_key_value_page(keyvalue_range: Dict[str, Any]) -> KeyValuePage:
    return KeyValuePage(
        key_value_pairs=[to_key_value(key_value) for key_value in keyvalue_range["kvs"]],
        next_key=_to_key(keyvalue_range["next_key"]),
    )


def _to_key(key: Optional[str]) -> Optional[Key]:
    if key is None:
        return None

    return Key(base64url_decode_to_bytes(key))


def to_key_value(key_value: Dict[str, Any]) -> KeyValue:
    return KeyValue(
        key=base64url_decode_to_bytes(key_value["key"]),
        value=base64url_decode_to_bytes(key_value["value"]),
    )


def to_key_value_string(key_value: KeyValue, encoding: str = "utf-8") -> KeyValueString:
    return KeyValueString(
        key=key_value.key.decode(encoding), value=key_value.value.decode(encoding)
    )
