import traceback
from typing import Any, Callable, Dict, List, Optional

from .app import AppEvent
from .task import TaskEvent


def format_exception(e: Optional[Exception]) -> Optional[List[str]]:
    if e is None:
        return None

    return traceback.format_exception(type(e), e, e.__traceback__)


def task_event_json(event: TaskEvent) -> Dict[str, Any]:
    return {
        "id": event.id,
        "input": event.input,
        "status": event.status,
        "output": event.output,
        "error": format_exception(event.error),
    }


def event_json(event: AppEvent) -> Dict[str, Any]:
    return {
        "id": event.id,
        "app_id": event.app_id,
        "input": event.input,
        "status": event.status,
        "output": event.output,
        "error": format_exception(event.error),
        "tasks": [task_event_json(task) for task in event.tasks],
    }


class EventHandler:
    def __init__(self) -> None:
        self.on_change_event: Callable[[Dict[str, Any]], None] = lambda x: None
        self.events: List[AppEvent] = []
        self.active_event: List[str] = ["none"]

    def set_event(self, callback: Callable[[Dict[str, Any]], None]) -> None:
        self.on_change_event = callback

    def on_change(self, message: Dict[str, Any]) -> None:
        self.on_change_event(message)

    def add_event(self, event: AppEvent) -> None:
        self.active_event[0] = event.id
        self.events.append(event)

        self.on_change_event({"type": "add_request", "payload": event_json(event)})

    def update_event(self, event: AppEvent) -> None:
        for i, e in enumerate(self.events):
            if e.id == self.active_event[0]:
                self.events[i] = event

                self.on_change_event({"type": "update_request", "payload": event_json(event)})
                break

    def task_event(self, task_event: TaskEvent) -> None:
        for i, event in enumerate(self.events):
            if event.id == self.active_event[0]:
                event.add_task_event(task_event)

                self.events[i] = event

                self.on_change_event({"type": "update_request", "payload": event_json(event)})
                break
