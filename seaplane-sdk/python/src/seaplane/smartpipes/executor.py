import json
from typing import Any

from ..logging import log
from ..model.errors import HTTPError
from .coprocessor import Coprocessor, CoprocessorEvent
from .event_handler import EventHandler


class CoprocessorExecutor:
    def execute(self, coprocessor: Coprocessor, *args: Any, **kwargs: Any) -> Any:
        pass


class RealCoProcessorExecutor(CoprocessorExecutor):
    def __init__(self, event_handler: EventHandler) -> None:
        self.event_handler = event_handler

    def execute(self, coprocessor: Coprocessor, *args: Any, **kwargs: Any) -> Any:
        args_str = tuple(arg.decode() if isinstance(arg, bytes) else arg for arg in args)
        args_json = json.dumps(args_str)

        event = CoprocessorEvent(coprocessor.id, args_json)
        self.event_handler.coprocessor_event(event)
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

        self.event_handler.coprocessor_event(event)

        if event.error is not None:
            raise event.error

        return result


class SchemaExecutor(CoprocessorExecutor):
    def __init__(self) -> None:
        ...

    def execute(self, coprocessor: Coprocessor, *args: Any, **kwargs: Any) -> Any:
        arguments = []
        for arg in args:
            arguments.append(arg)

        coprocessor.called_from(arguments)
        return coprocessor.id
