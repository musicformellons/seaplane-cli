import functools
from typing import Any, Dict

from flask import Flask, request
from flask_cors import CORS
from flask_socketio import SocketIO, emit

from ..configuration import config
from ..logging import log
from .decorators import context, smart_pipes_json
from .smartpipe import SmartPipe

app = Flask(__name__)
sio = SocketIO(app, cors_allowed_origins=["http://localhost:3000"], async_mode="threading")
CORS(app, origins=["http://localhost:3000"])


@sio.on("message")  # type: ignore
def handle_message(data: Any) -> None:
    print("received message:", data)


@sio.on("connect")  # type: ignore
def on_connect() -> None:
    emit("message", smart_pipes_json(context.smart_pipes))


def send_something(data: Any) -> None:
    emit("message", data, sid="sp", namespace="", broadcast=True)


def start() -> Flask:
    context.set_event(lambda data: send_something(data))

    smart_pipes = context.smart_pipes

    for smart_pipe in smart_pipes:

        def endpoint_func(pipe: SmartPipe = smart_pipe) -> Dict[str, Any]:
            data = request.get_json()
            result = pipe.func(data)
            return {"result": result}

        endpoint = functools.partial(endpoint_func, pipe=smart_pipe)
        app.add_url_rule(smart_pipe.path, smart_pipe.id, endpoint, methods=[smart_pipe.method])

    def health() -> str:
        emit("message", {"data": "test"}, sid="lol", namespace="", broadcast=True)
        return "Seaplane SmartPipes Demo"

    app.add_url_rule("/", "health", health, methods=["GET"])

    if not config.is_production():
        log.info("🚀 Smart Pipes in DEVELOPMENT MODE")
        sio.run(app, debug=False, port=1337)
    else:
        log.info("🚀 Smart Pipes in PRODUCTION")

    return app
