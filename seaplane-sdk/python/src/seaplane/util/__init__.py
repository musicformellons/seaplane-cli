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
