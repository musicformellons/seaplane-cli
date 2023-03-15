from seaplane.model.metadata import (
    KeyString,
    KeyValue,
    KeyValueStream,
    KeyValueString,
    to_key_value_string,
)

from ...util import get_absolute_path, get_file_bytes


def test_key_string_encode_to_base64url() -> None:
    key_value_string = KeyString("foo/bar")
    assert key_value_string.key == b"foo/bar"
    assert key_value_string.key_str == "foo/bar"


def test_key_value_string_encode_to_base64url() -> None:
    key_value_string = KeyValueString("foo/bar", "bar")
    assert key_value_string.key == b"foo/bar"
    assert key_value_string.value == b"bar"
    assert key_value_string.key_str == "foo/bar"
    assert key_value_string.value_str == "bar"


def test_key_value_is_encoded_to_key_value_string() -> None:
    key_value = KeyValue(b"foo/bar", b"bar")

    key_value_string = to_key_value_string(key_value)
    assert key_value_string.key == b"foo/bar"
    assert key_value_string.value == b"bar"
    assert key_value_string.key_str == "foo/bar"
    assert key_value_string.value_str == "bar"


def test_key_value_stream_reads_value_text_content() -> None:
    config_path = get_absolute_path("fixtures/metadata/config.toml")
    key_value_stream = KeyValueStream(b"key", open(config_path, "rb"))

    assert key_value_stream.key == b"key"
    assert (
        key_value_stream.value
        == b'[tool.poetry]\nname = "example configuration"\nversion = "0.1"\ndescription = "Seaplane Python SDK example config"\nauthors = ["Seaplane IO, Inc."]\nlicense = "Apache License"\nreadme = "README.md"\nrepository = "https://github.com/seaplane-io"\ndocumentation = "https://github.com/seaplane-io"'  # noqa: E501
    )


def test_key_value_stream_reads_value_image_content() -> None:
    image_path = get_absolute_path("fixtures/metadata/seaplane.jpeg")
    key_value_stream = KeyValueStream(b"key", open(image_path, "rb"))

    image_content = get_file_bytes(absolute_path=image_path)

    assert key_value_stream.key == b"key"
    assert len(key_value_stream.value) == 12565
    assert key_value_stream.value == image_content
