import json
import os
from typing import Any, Dict

from .decorators import context
from .executor import SchemaExecutor


def persist_schema(schema: Dict[str, Any]) -> None:
    if not os.path.exists("build"):
        os.makedirs("build")

    file_path = os.path.join("build", "workload.json")

    with open(file_path, "w") as file:
        json.dump(schema, file, indent=2)


def build() -> None:
    schema: Dict[str, Any] = {"smartpipes": []}

    context.set_executor(SchemaExecutor())

    for sm in context.smart_pipes:
        print(f"Smart Pipe {sm.id} : ")
        smartpipe: Dict[str, Any] = {
            "id": sm.id,
            "entry_point": {"type": "API", "path": sm.path, "method": sm.method},
            "coprocessors": [],
            "io": {},
        }

        sm.func("entry_point")
        for c in sm.coprocessors:
            coprocessor = {"id": c.id, "name": c.name, "type": c.type, "model": c.model}

            for source in c.sources:
                if not smartpipe["io"].get(source, None):
                    smartpipe["io"][source] = [c.id]
                else:
                    smartpipe["io"][source].append(c.id)

            smartpipe["coprocessors"].append(coprocessor)

            print(f"Coprocessor {c.id} source {c.sources}")

        schema["smartpipes"].append(smartpipe)

    persist_schema(schema)
