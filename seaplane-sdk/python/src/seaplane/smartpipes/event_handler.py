import traceback
from typing import Any, Callable, Dict, List, Optional

from .coprocessor import CoprocessorEvent
from .smartpipe import SmartPipeEvent


def format_exception(e: Optional[Exception]) -> Optional[List[str]]:
    if e is None:
        return None

    return traceback.format_exception(type(e), e, e.__traceback__)


def coprocessor_event_json(event: CoprocessorEvent) -> Dict[str, Any]:
    return {
        "id": event.id,
        "input": event.input,
        "status": event.status,
        "output": event.output,
        "error": format_exception(event.error),
    }


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


class EventHandler:
    def __init__(self) -> None:
        self.on_change_event: Callable[[Dict[str, Any]], None] = lambda x: None
        self.events: List[SmartPipeEvent] = []
        self.active_event: List[str] = ["none"]

    def set_event(self, callback: Callable[[Dict[str, Any]], None]) -> None:
        self.on_change_event = callback

    def on_change(self, message: Dict[str, Any]) -> None:
        self.on_change_event(message)

    def add_event(self, event: SmartPipeEvent) -> None:
        self.active_event[0] = event.id
        self.events.append(event)

        self.on_change_event({"type": "add_request", "payload": event_json(event)})

    def update_event(self, event: SmartPipeEvent) -> None:
        for i, e in enumerate(self.events):
            if e.id == self.active_event[0]:
                self.events[i] = event

                self.on_change_event({"type": "update_request", "payload": event_json(event)})
                break

    def coprocessor_event(self, coprocessor_event: CoprocessorEvent) -> None:
        for i, event in enumerate(self.events):
            if event.id == self.active_event[0]:
                event.add_coprocessor_event(coprocessor_event)

                self.events[i] = event

                self.on_change_event({"type": "update_request", "payload": event_json(event)})
                break
