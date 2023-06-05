import functools
import json
import traceback
from typing import Any, Callable, Dict, List, Optional

from ..logging import log
from ..model.errors import HTTPError
from .coprocessor import Coprocessor, CoprocessorEvent
from .smartpipe import SmartPipe, SmartPipeEvent


def format_exception(e: Optional[Exception]) -> Optional[List[str]]:
    if e is None:
        return None

    return traceback.format_exception(type(e), e, e.__traceback__)


def event_json(event: SmartPipeEvent) -> Dict[str, Any]:
    return {
        "id": event.id,
        "smart_pipe_id": event.smart_pipe_id,
        "input": event.input,
        "status": event.status,
        "output": event.output,
        "error": format_exception(event.error),
        "coprocessors": [
            coprocessor_event_json(coprocessor) for coprocessor in event.coprocessors
        ],
    }


def coprocessor_event_json(event: CoprocessorEvent) -> Dict[str, Any]:
    return {
        "id": event.id,
        "input": event.input,
        "status": event.status,
        "output": event.output,
        "error": format_exception(event.error),
    }


def coprocessor_to_json(coprocessor: Coprocessor) -> Dict[str, Any]:
    return {"id": coprocessor.id, "type": coprocessor.type, "model": coprocessor.model}


def smart_pipe_to_json(smart_pipe: SmartPipe) -> Dict[str, Any]:
    return {
        "id": smart_pipe.id,
        "path": smart_pipe.path,
        "method": smart_pipe.method,
        "coprocessors": [
            coprocessor_to_json(coprocessor) for coprocessor in smart_pipe.coprocessors
        ],
    }


def smart_pipes_json(smart_pipes: List[SmartPipe]) -> Dict[str, Any]:
    return {
        "type": "smart_pipes",
        "payload": [smart_pipe_to_json(smart_pipe) for smart_pipe in smart_pipes],
    }


class Context:
    def __init__(
        self,
        smart_pipes: Optional[List[SmartPipe]] = None,
        coprocessors: Optional[List[Coprocessor]] = None,
    ) -> None:
        self.actual_smart_pipe_index = -1
        self.on_change: Callable[[Dict[str, Any]], None] = lambda x: None
        self.events: List[SmartPipeEvent] = []
        self.active_event: List[str] = ["none"]

        if smart_pipes is None:
            self.smart_pipes = []
        else:
            self.smart_pipes = smart_pipes

        if coprocessors is None:
            self.coprocessors = []
        else:
            self.coprocessors = coprocessors

    def active_smart_pipe(self, id: str) -> None:
        for i, smart_pipe in enumerate(self.smart_pipes):
            if smart_pipe.id == id:
                self.actual_smart_pipe_index = i
                break

    def set_event(self, callback: Callable[[Dict[str, Any]], None]) -> None:
        self.on_change = callback

    def add_event(self, event: SmartPipeEvent) -> None:
        self.active_event[0] = event.id
        self.events.append(event)

        self.on_change({"type": "add_request", "payload": event_json(event)})

    def update_event(self, event: SmartPipeEvent) -> None:
        for i, e in enumerate(self.events):
            if e.id == self.active_event[0]:
                self.events[i] = event

                self.on_change({"type": "update_request", "payload": event_json(event)})
                break

    def coprocessor_event(self, coprocessor_event: CoprocessorEvent) -> None:
        for i, event in enumerate(self.events):
            if event.id == self.active_event[0]:
                event.add_coprocessor_event(coprocessor_event)

                self.events[i] = event

                self.on_change({"type": "update_request", "payload": event_json(event)})
                break

    def get_actual_smart_pipe(self) -> Optional[SmartPipe]:
        if self.actual_smart_pipe_index == -1:
            return None

        return self.smart_pipes[self.actual_smart_pipe_index]

    def add_smart_pipe(self, smart_pipe: SmartPipe) -> None:
        if len(self.smart_pipes) == 1 and self.smart_pipes[0].id == "temporal":
            self.smart_pipes[0] = smart_pipe
        else:
            self.actual_smart_pipe_index = len(self.smart_pipes)
            self.smart_pipes.append(smart_pipe)

        log.info(f"🧠 Smart Pipe: {smart_pipe.id}, Path: {smart_pipe.path}")
        self.on_change(smart_pipes_json(self.smart_pipes))

    def add_coprocessor(self, coprocessor: Coprocessor) -> None:
        log.info(f"⌛️ Coprocessor {coprocessor.id} of type: {coprocessor.type}")
        self.coprocessors.append(coprocessor)

    def assign_to_active_smart_pipe(self, coprocessor: Coprocessor) -> None:
        self.smart_pipes[self.actual_smart_pipe_index].add_coprocessor(coprocessor)
        smart_pipe = context.get_actual_smart_pipe()
        if smart_pipe is not None:
            log.info(f"⌛️ Assign Coprocessor {coprocessor.id} to Smart Pipe: {smart_pipe.id}")
        else:
            log.info(
                f"🔥 Actual Smart Pipe is None, can't assign Coprocessor {coprocessor.id} to Smart Pipe"  # noqa
            )
        self.on_change(smart_pipes_json(self.smart_pipes))


context = Context()


def smartpipe(
    path: str, method: str, id: str, _func: Optional[Callable[[Any], Any]] = None
) -> Callable[[Any, Any], Any]:
    def decorator_smartpipes(func: Callable[[Any], Any]) -> Callable[[Any, Any], Any]:
        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            print(f"SmartPipe Path: {path}, Method: {method}, ID: {id}")
            context.active_smart_pipe(id)

            args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
            args_json = json.dumps(args_str)

            event = SmartPipeEvent(smart_pipe_id=id, input=args_json)
            context.add_event(event)
            result = None

            try:
                result = func(*args, **kwargs)
                event.set_ouput(result)
            except HTTPError as err:
                log.error(f"Smart Pipe error: {err}")
                event.set_error(err)

            context.update_event(event)
            return result

        smart_pipe = SmartPipe(wrapper, path, method, id)
        context.add_smart_pipe(smart_pipe)
        return wrapper

    if not _func:
        return decorator_smartpipes  # type: ignore
    else:
        return decorator_smartpipes(_func)


def import_coprocessor(
    _func: Optional[Callable[[Any], Any]], coprocessor: Coprocessor
) -> Callable[[Any, Any], Any]:
    def decorator_coprocessor(func: Callable[[Any], Any]) -> Callable[[Any, Any], Any]:
        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
            args_json = json.dumps(args_str)

            event = CoprocessorEvent(coprocessor.id, args_json)
            context.coprocessor_event(event)

            result = coprocessor.process(*args, **kwargs)

            event.set_ouput(result)
            context.coprocessor_event(event)

            return func(result)

        return wrapper

    context.add_coprocessor(coprocessor)

    if not _func:
        return decorator_coprocessor  # type: ignore
    else:
        return decorator_coprocessor(_func)


def coprocessor(
    type: str, id: str, model: Optional[str] = None, _func: Optional[Callable[[Any], Any]] = None,
) -> Callable[[Any, Any], Any]:
    def decorator_coprocessor(func: Callable[[Any], Any]) -> Callable[[Any, Any], Any]:

        coprocessor = Coprocessor(func, type, model, id)
        context.add_coprocessor(coprocessor)

        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            context.assign_to_active_smart_pipe(coprocessor)

            args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
            args_json = json.dumps(args_str)

            event = CoprocessorEvent(coprocessor.id, args_json)
            context.coprocessor_event(event)
            result = None

            try:
                result = coprocessor.process(*args, **kwargs)
                event.set_ouput(result)
            except HTTPError as err:
                log.error(f"Coprocessor HTTPError: {err}")
                event.set_error(err)
            except Exception as e:
                log.error(f"Coprocessor error: {e}")
                event.set_error(e)

            context.coprocessor_event(event)

            if event.error is not None:
                raise event.error

            return result

        return wrapper

    if not _func:
        return decorator_coprocessor  # type: ignore
    else:
        return decorator_coprocessor(_func)
