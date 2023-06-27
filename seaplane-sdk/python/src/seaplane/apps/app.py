import secrets
import string
from typing import Any, Callable, List, Optional

from ..logging import log
from .task import Task, TaskEvent


def random_id() -> str:
    alphabet = string.ascii_letters + string.digits
    random_string = "".join(secrets.choice(alphabet) for i in range(10))
    return random_string


class AppEvent:
    def __init__(self, app_id: str, input: Any) -> None:
        self.id = random_id()
        self.app_id = app_id
        self.status = "in_progress"
        self.input: Any = input
        self.output: Optional[Any] = None
        self.tasks: List[TaskEvent] = []
        self.error: Optional[Any] = None

    def add_task_event(self, event: TaskEvent) -> None:
        for i, cp in enumerate(self.tasks):
            if cp.id == event.id:
                self.tasks[i] = event
                return
        self.tasks.append(event)

    def set_output(self, output: Any) -> None:
        self.output = output
        self.status = "completed"

    def set_error(self, error: Any) -> None:
        self.error = error
        self.status = "error"


class App:
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
        self.tasks: List[Task] = []
        self.events: List[AppEvent] = []
        self.return_source = None

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.func(*args, *kwargs)

    def add_task(self, task: Task) -> None:
        for i, cp in enumerate(self.tasks):
            if cp.id == task.id:
                self.tasks[i] = task
                return
        self.tasks.append(task)

    def print(self) -> None:
        log.info(f"id: {self.id}, path: {self.path}, method: {self.method}")
