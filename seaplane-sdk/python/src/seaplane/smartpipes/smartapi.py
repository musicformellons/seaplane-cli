import functools
import os
from typing import Any

from flask import Flask, request
from flask_cors import CORS
from flask_socketio import SocketIO, emit

from ..configuration import config
from ..logging import log
from .decorators import context, smart_pipes_json
from .smartpipe import SmartPipe

SMARTPIPES_CORS = list(os.getenv("SMARTPIPES_CORS", "http://localhost:3000").split(" "))
AUTH_TOKEN = os.getenv("SMARTPIPES_AUTH_TOKEN")

app = Flask(__name__)
sio = SocketIO(app, cors_allowed_origins=SMARTPIPES_CORS, async_mode="threading")
CORS(app, origins=SMARTPIPES_CORS)


@sio.on("message")  # type: ignore
def handle_message(data: Any) -> None:
    ...


@sio.on("connect")  # type: ignore
def on_connect() -> None:
    emit("message", smart_pipes_json(context.smart_pipes))


def send_something(data: Any) -> None:
    emit("message", data, sid="sp", namespace="", broadcast=True)


def authenticate_token() -> bool:
    token = request.headers.get("Authorization")
    if token and token.startswith("Bearer "):

        if token == f"Bearer {AUTH_TOKEN}":
            return True

    return False


@app.before_request
def before_request() -> Any:
    if request.path == "/healthz":
        return

    if os.getenv("SMARTPIPES_AUTH_TOKEN") is not None:
        if not authenticate_token():
            return {"message": "Unauthorized"}, 401


def start() -> Flask:
    log.debug(f"CORS enabled: {SMARTPIPES_CORS}")
    context.set_event(lambda data: send_something(data))

    smart_pipes = context.smart_pipes

    for smart_pipe in smart_pipes:

        def endpoint_func(pipe: SmartPipe = smart_pipe) -> Any:
            data = request.get_json()
            result = pipe.func(data)
            return result

        endpoint = functools.partial(endpoint_func, pipe=smart_pipe)
        app.add_url_rule(smart_pipe.path, smart_pipe.id, endpoint, methods=[smart_pipe.method])

    def health() -> str:
        emit("message", {"data": "test"}, sid="lol", namespace="", broadcast=True)
        return "Seaplane SmartPipes Demo"

    app.add_url_rule("/", "healthz", health, methods=["GET"])

    if not config.is_production():
        log.info("🚀 Smart Pipes DEVELOPMENT MODE")
        sio.run(app, debug=False, port=1337)
    else:
        log.info("🚀 Smart Pipes PRODUCTION MODE")

    return app
