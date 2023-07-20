from seaplane import app, config, log, start, task
from seaplane.logging import SeaLogger

config.set_global_sql_endpoint("https://sql.staging.cplane.dev/v1")
config.set_carrier_endpoint("https://carrier.staging.cplane.dev/v1")

config.set_api_key("<YOUR_SEAPLANE_KEY>")


@task(type="compute", id="hello-world-task")
def hello_world_task(data):
    return "hello world"


@app(id="hello-world-app", path="/hello")
def hello_world_app(data):
    return hello_world_task(data)


start()
