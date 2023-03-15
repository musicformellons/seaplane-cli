from base64 import urlsafe_b64decode, urlsafe_b64encode


def base64url_encode_from_bytes(data: bytes) -> str:
    return urlsafe_b64encode(data).rstrip(b"=").decode()


def base64url_decode_to_bytes(data: str) -> bytes:
    padding = b"=" * (4 - (len(data) % 4))
    return urlsafe_b64decode(data.encode() + padding)
