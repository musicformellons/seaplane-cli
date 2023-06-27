import json
from typing import Any

from ..logging import log
from ..model.errors import HTTPError
from .app import App
from .event_handler import EventHandler
from .task import Task, TaskEvent


class TaskExecutor:
    def execute(self, task: Task, *args: Any, **kwargs: Any) -> Any:
        pass


class RealTaskExecutor(TaskExecutor):
    def __init__(self, event_handler: EventHandler) -> None:
        self.event_handler = event_handler

    def execute(self, task: Task, *args: Any, **kwargs: Any) -> Any:
        args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
        args_json = json.dumps(args_str)

        event = TaskEvent(task.id, args_json)
        self.event_handler.task_event(event)
        result = None

        try:
            result = task.process(*args, **kwargs)
            event.set_output(result)
        except HTTPError as err:
            log.error(f"Task HTTPError: {err}")
            event.set_error(err)
        except Exception as e:
            log.error(f"Task error: {e}")
            event.set_error(e)

        self.event_handler.task_event(event)

        if event.error is not None:
            raise event.error

        return result


class SchemaExecutor(TaskExecutor):
    def __init__(self) -> None:
        ...

    def execute(self, task: Task, *args: Any, **kwargs: Any) -> Any:
        arguments = []
        for arg in args:
            arguments.append(arg)

        task.called_from(arguments)
        return task.id


class AppExecutor:
    def execute(self, app: App, *args: Any, **kwargs: Any) -> Any:
        pass


class ProductionAppExecutor(AppExecutor):
    def __init__(self) -> None:
        ...

    def execute(self, app: App, *args: Any, **kwargs: Any) -> Any:
        pass


class DevelopmentAppExecutor(AppExecutor):
    def __init__(self) -> None:
        ...

    def execute(self, app: App, *args: Any, **kwargs: Any) -> Any:
        pass
