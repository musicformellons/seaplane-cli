import asyncio
import functools
import json
import os
import sys
import traceback
from typing import Any, List, Optional

from flask import Flask, request
from flask_cors import CORS
from flask_socketio import SocketIO, emit
import nats

from ..carrier import processor
from ..configuration import config
from ..logging import log
from ..model.errors import SeaplaneError
from .build import build
from .decorators import context, smart_pipes_json
from .deploy import deploy, destroy
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


def dev_https_api_start() -> Flask:
    log.debug(f"CORS enabled: {SMARTPIPES_CORS}")
    context.set_event(lambda data: send_something(data))

    smart_pipes = context.smart_pipes

    for smart_pipe in smart_pipes:

        def endpoint_func(pipe: SmartPipe = smart_pipe) -> Any:
            if request.method == "POST" or request.method == "PUT":
                data = request.get_json()
                result = pipe.func(data)
                return result
            elif request.method == "GET":
                return pipe.func("nothing")  # current limitation, it needs to pass something

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


loop = asyncio.get_event_loop()


async def publish_message(stream: str, message: Any, coprocessors: List[str]) -> None:
    nc = await nats.connect(
        ["nats://carrier.staging.cplane.dev:2003"],
        user_credentials=os.path.abspath("./carrier.creds"),
    )
    js = nc.jetstream()

    nats_message = {"input": message}

    for coprocessor in coprocessors:
        log.debug(f"Sending to {stream}.{coprocessor} coprocessor,  message: {nats_message}")
        ack = await js.publish(f"{stream}.{coprocessor}", str(json.dumps(nats_message)).encode())
        log.debug(f"ACK: {ack}")

    await nc.close()


def prod_https_api_start() -> Flask:
    log.debug(f"CORS enabled: {SMARTPIPES_CORS}")

    schema = build()
    context.set_event(lambda data: send_something(data))

    smart_pipes = context.smart_pipes

    for smart_pipe in smart_pipes:

        def endpoint_func(pipe: SmartPipe = smart_pipe) -> Any:
            if request.method == "POST" or request.method == "PUT":
                data = request.get_json()
                smart_pipe_first_coprocessors = schema["smartpipes"][pipe.id]["io"]["entry_point"]
                loop.run_until_complete(
                    publish_message(pipe.id, data, smart_pipe_first_coprocessors)
                )
                return "Ok"
            elif request.method == "GET":
                return pipe.func("nothing")  # current limitation, it needs to pass something

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


def start_coprocessor(coprocessor_id: str) -> None:
    coprocessor = context.get_coprocessor(coprocessor_id)

    if not coprocessor:
        raise SeaplaneError(
            f"Coprocessor {coprocessor_id} not found, \
                            make sure the Coprocessor ID is correct."
        )

    processor.start()

    while True:
        try:
            log.info(f" Coprocessor {coprocessor.id}  waiting for getting data...")
            message = json.loads(processor.read())
            log.debug(f" Message received: {message}")

            message["output"] = coprocessor.process(message["input"])

            log.debug(f" Coprocessor Result: {message}")

            next_message = {"input": message["output"]}
            processor.write(str(json.dumps(next_message)).encode())
        except Exception as e:
            log.error(
                f"Error running Coprocessor:\
                      \n {traceback.format_exception(type(e), e, e.__traceback__)}"
            )


def start() -> Optional[Flask]:
    if len(sys.argv) > 1:
        command = sys.argv[1]

        if command == "build":
            build()
            return None
        elif command == "deploy":
            if len(sys.argv) == 3:
                co_id = sys.argv[2]
                deploy(co_id)
            else:
                deploy()
            return None
        elif command == "destroy":
            destroy()
            return None

    coprocessor_id: Optional[str] = os.getenv("COPROCESSOR_ID")

    if not coprocessor_id:
        log.info("Starting API Entry Point...")
        if not config.is_production():
            return dev_https_api_start()
        else:
            return prod_https_api_start()
    else:
        log.info(f"Starting Coprocessor {coprocessor_id} ...")
        start_coprocessor(coprocessor_id)
        return None
