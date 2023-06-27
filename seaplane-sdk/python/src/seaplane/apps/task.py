from typing import Any, Callable, Dict, List, Optional, Tuple

from ..logging import log
from ..model.errors import SeaplaneError
from .tasks import Bloom, OpenAI, Replicate, Sql, Store


class TaskEvent:
    def __init__(self, id: str, input: Any) -> None:
        self.id = id
        self.status = "in_progress"
        self.input = input
        self.output: Optional[Any] = None
        self.error: Optional[Any] = None

    def set_output(self, output: Any) -> None:
        self.output = output
        self.status = "completed"

    def set_error(self, error: Any) -> None:
        self.error = error
        self.status = "error"


SEAPLANE_API_KEY_NAME = "SEAPLANE_API_KEY"
OPENAI_API_KEY_NAME = "OPENAI_API_KEY"
REPLICATE_API_KEY_NAME = "REPLICATE_API_KEY"


class Task:
    def __init__(
        self,
        func: Callable[[Any], Any],
        type: str,
        id: Optional[str] = None,
        model: Optional[str] = None,
        sql_access: Optional[Dict[str, str]] = None,
    ) -> None:
        self.func = func
        self.args: Optional[Tuple[Any, ...]] = None
        self.kwargs: Optional[Dict[str, Any]] = None
        self.type = type
        self.model = model
        self.sources: List[str] = []
        self.sql_access = sql_access
        self.sql: Optional[Sql] = None
        self.name = func.__name__

        if id is not None:
            self.id = id
        else:
            self.id = func.__name__

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        log.info(f"Task type '{self.type}' Model ID {self.model}")

        if self.type == "sql":
            if self.sql_access is None:
                raise SeaplaneError("Task of type SQL without sql attribute.")

            if not self.sql:
                self.sql = Sql(self.func, self.id, self.sql_access)

            return self.sql.process(*self.args, **self.kwargs)

        if self.type == "vectordb":
            log.info("Accessing Vector DB task...")
            self.args = self.args + (Store(),)
            return self.func(*self.args, **self.kwargs)

        if self.model == "bloom":
            bloom = Bloom(self.func, self.id, self.model)
            return bloom.process(*self.args, **self.kwargs)
        elif self.model == "gpt-3.5":
            openai = OpenAI(self.func, self.id, self.model)
            return openai.process(*self.args, **self.kwargs)
        elif self.model == "gpt-3":
            openai = OpenAI(self.func, self.id, self.model)
            return openai.process(*self.args, **self.kwargs)
        elif self.model == "stable-diffusion":
            replicate = Replicate(self.func, self.type, self.id, self.model)
            return replicate.process(*self.args, **self.kwargs)
        elif self.model:
            replicate = Replicate(self.func, self.type, self.id, self.model)
            return replicate.process(*self.args, **self.kwargs)
        else:
            log.info("Compute task type...")
            return self.func(*self.args, **self.kwargs)

    def called_from(self, sources: List[str]) -> None:
        self.sources = sources

    def print(self) -> None:
        log.info(f"id: {self.id}, type: {self.type}, model: {self.model}")
