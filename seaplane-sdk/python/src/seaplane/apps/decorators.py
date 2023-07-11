import functools
import json
from typing import Any, Callable, Dict, List, Optional

from ..logging import log
from ..model.errors import HTTPError
from .app import App, AppEvent
from .event_handler import EventHandler
from .executor import RealTaskExecutor, TaskExecutor
from .task import Task, TaskEvent


def task_to_json(task: Task) -> Dict[str, Any]:
    return {"id": task.id, "type": task.type, "model": task.model}


def app_to_json(app: App) -> Dict[str, Any]:
    return {
        "id": app.id,
        "path": app.path,
        "method": app.method,
        "tasks": [task_to_json(task) for task in app.tasks],
    }


def apps_json(apps: List[App]) -> Dict[str, Any]:
    return {
        "type": "apps",
        "payload": [app_to_json(app) for app in apps],
    }


class Context:
    def __init__(
        self,
        apps: Optional[List[App]] = None,
        tasks: Optional[List[Task]] = None,
    ) -> None:
        self.actual_app_index = -1
        self.event_handler = EventHandler()
        self.task_executor: TaskExecutor = RealTaskExecutor(self.event_handler)

        if apps is None:
            self.apps = []
        else:
            self.apps = apps

        if tasks is None:
            self.tasks = []
        else:
            self.tasks = tasks

    def active_app(self, id: str) -> None:
        for i, app in enumerate(self.apps):
            if app.id == id:
                self.actual_app_index = i
                break

    def set_event(self, callback: Callable[[Dict[str, Any]], None]) -> None:
        self.event_handler.set_event(callback)

    def add_event(self, event: AppEvent) -> None:
        self.event_handler.add_event(event)

    def update_event(self, event: AppEvent) -> None:
        self.event_handler.update_event(event)

    def task_event(self, task_event: TaskEvent) -> None:
        self.event_handler.task_event(task_event)

    def get_actual_app(self) -> Optional[App]:
        if self.actual_app_index == -1:
            return None

        return self.apps[self.actual_app_index]

    def add_app(self, app: App) -> None:
        if len(self.apps) == 1 and self.apps[0].id == "temporal":
            self.apps[0] = app
        else:
            self.actual_app_index = len(self.apps)
            self.apps.append(app)

        log.debug(f"🧠 App: {app.id}, Path: {app.path}")
        self.event_handler.on_change(apps_json(self.apps))

    def add_task(self, task: Task) -> None:
        log.debug(f"⌛️ Task {task.id} of type: {task.type}")
        self.tasks.append(task)

    def get_task(self, id: str) -> Optional[Task]:
        for c in self.tasks:
            if c.id == id:
                return c

        return None

    def assign_to_active_app(self, task: Task) -> None:
        self.apps[self.actual_app_index].add_task(task)
        app = context.get_actual_app()
        if app is not None:
            log.info(f"⌛️ Assign Task {task.id} to App: {app.id}")
        else:
            log.info(
                f"🔥 Actual App is None, can't assign \
                    Task {task.id} to App"
            )
        self.event_handler.on_change(apps_json(self.apps))

    def set_executor(self, executor: TaskExecutor) -> None:
        self.task_executor = executor


context = Context()


def app(
    path: str,
    id: str,
    method: str = "POST",
    parameters: Optional[List[str]] = None,
    _func: Optional[Callable[[Any], Any]] = None,
) -> Callable[[Any, Any], Any]:
    def decorator_apps(func: Callable[[Any], Any]) -> Callable[[Any, Any], Any]:
        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            log.debug(f"App Path: {path}, Method: {method}, ID: {id}")
            context.active_app(id)

            args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
            args_json = json.dumps(args_str)

            event = AppEvent(app_id=id, input=args_json)
            context.add_event(event)
            result = None

            try:
                result = func(*args, **kwargs)
                event.set_output(result)
            except HTTPError as err:
                log.error(f"App error: {err}")
                event.set_error(err)

            context.update_event(event)
            return result

        app = App(wrapper, path, method, id, parameters)
        context.add_app(app)
        return wrapper

    if not _func:
        return decorator_apps  # type: ignore
    else:
        return decorator_apps(_func)


def import_task(_func: Optional[Callable[[Any], Any]], task: Task) -> Callable[[Any, Any], Any]:
    def decorator_task(func: Callable[[Any], Any]) -> Callable[[Any, Any], Any]:
        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
            args_json = json.dumps(args_str)

            event = TaskEvent(task.id, args_json)
            context.task_event(event)

            result = task.process(*args, **kwargs)

            event.set_output(result)
            context.task_event(event)

            return func(result)

        return wrapper

    context.add_task(task)

    if not _func:
        return decorator_task  # type: ignore
    else:
        return decorator_task(_func)


def task(
    type: str,
    id: Optional[str] = None,
    model: Optional[str] = None,
    sql: Optional[Dict[str, str]] = None,
    _func: Optional[Callable[[Any], Any]] = None,
) -> Callable[[Any, Any], Any]:
    def decorator_task(func: Callable[[Any], Any]) -> Callable[[Any, Any], Any]:

        task = Task(func=func, type=type, model=model, id=id, sql_access=sql)
        context.add_task(task)

        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            context.assign_to_active_app(task)

            return context.task_executor.execute(task, *args, **kwargs)

        return wrapper

    if not _func:
        return decorator_task  # type: ignore
    else:
        return decorator_task(_func)
