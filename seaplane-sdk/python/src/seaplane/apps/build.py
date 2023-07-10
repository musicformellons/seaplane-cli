import json
import os
from typing import Any, Dict

from ..logging import log
from .decorators import context
from .executor import RealTaskExecutor, SchemaExecutor


def persist_schema(schema: Dict[str, Any]) -> None:
    if not os.path.exists("build"):
        os.makedirs("build")

    file_path = os.path.join("build", "schema.json")

    with open(file_path, "w") as file:
        json.dump(schema, file, indent=2)


def build() -> Dict[str, Any]:
    schema: Dict[str, Any] = {"apps": {}}

    context.set_executor(SchemaExecutor())

    for sm in context.apps:
        result = sm.func("entry_point")
        sm.return_source = result

    for sm in context.apps:
        app: Dict[str, Any] = {
            "id": sm.id,
            "entry_point": {"type": "API", "path": sm.path, "method": sm.method},
            "tasks": [],
            "io": {},
        }

        for c in sm.tasks:
            task = {"id": c.id, "name": c.name, "type": c.type, "model": c.model}

            for source in c.sources:
                if not app["io"].get(source, None):
                    app["io"][source] = [c.id]
                else:
                    app["io"][source].append(c.id)

            app["tasks"].append(task)

        app["io"]["returns"] = sm.return_source
        schema["apps"][sm.id] = app

    persist_schema(schema)

    log.debug("Apps build configuration done")

    context.set_executor(RealTaskExecutor(context.event_handler))

    return schema
