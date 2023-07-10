# noqa

from typing import TypeVar

from returns.pipeline import is_successful
from returns.result import Result

from ..model.errors import SeaplaneError

T = TypeVar("T")


def unwrap(result: Result[T, SeaplaneError]) -> T:
    if is_successful(result):
        try:
            return result.unwrap()
        except Exception as error:
            raise SeaplaneError(str(error))
    else:
        raise result.failure()


def remove_prefix(text: str, prefix: str) -> str:
    if text.startswith(prefix):
        return text[len(prefix) :]
    return text
