from seaplane import app, log, start, task
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)


@task(type="compute", id="hello-world-task")
def hello_world_task(data):
    return "hello world"


@app(id="hello-world-app", path="/hello")
def hello_world_app(data):
    return hello_world_task(data)


start()
