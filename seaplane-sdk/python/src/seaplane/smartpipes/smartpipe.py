import secrets
import string
from typing import Any, Callable, List, Optional

from ..logging import log
from .coprocessor import Coprocessor, CoprocessorEvent


def random_id() -> str:
    alphabet = string.ascii_letters + string.digits
    random_string = "".join(secrets.choice(alphabet) for i in range(10))
    return random_string


class SmartPipeEvent:
    def __init__(self, smart_pipe_id: str, input: Any) -> None:
        self.id = random_id()
        self.smart_pipe_id = smart_pipe_id
        self.status = "in_progress"
        self.input: Any = input
        self.output: Optional[Any] = None
        self.coprocessors: List[CoprocessorEvent] = []
        self.error: Optional[Any] = None

    def add_coprocessor_event(self, event: CoprocessorEvent) -> None:
        for i, cp in enumerate(self.coprocessors):
            if cp.id == event.id:
                self.coprocessors[i] = event
                return
        self.coprocessors.append(event)

    def set_ouput(self, output: Any) -> None:
        self.output = output
        self.status = "completed"

    def set_error(self, error: Any) -> None:
        self.error = error
        self.status = "error"


class SmartPipe:
    def __init__(
        self,
        func: Callable[[Any], Any],
        path: str,
        method: str,
        id: str,
        parameters: Optional[List[str]],
    ) -> None:
        self.func = func
        self.path = path
        self.method = method
        self.id = id
        self.parameters = parameters
        self.coprocessors: List[Coprocessor] = []
        self.events: List[SmartPipeEvent] = []

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.func(*args, *kwargs)

    def add_coprocessor(self, coprocessor: Coprocessor) -> None:
        for i, cp in enumerate(self.coprocessors):
            if cp.id == coprocessor.id:
                self.coprocessors[i] = coprocessor
                return
        self.coprocessors.append(coprocessor)

    def print(self) -> None:
        log.info(f"id: {self.id}, path: {self.path}, method: {self.method}")
