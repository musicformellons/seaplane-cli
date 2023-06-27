from seaplane import Task, app, config, context, import_task, log, start, task
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)
config.set_production(True)


@task(type="compute", id="string_to_int")
def toInt(input):

    return int(input)


@task(type="compute", id="multiply_by_1_8")
def multiply_by_1_8(number):

    return number * 1.8


@task(type="compute", id="add_32")
def add_32(number):

    return number + 32


@app(path="/celsius_to_fahrenheit", method="POST", id="celsius_to_fahrenheit")
def celsius_to_fahrenheit(input):

    number = toInt(input)
    temp_times_1_8 = multiply_by_1_8(number)
    temp_fahrenheit = add_32(temp_times_1_8)
    return temp_fahrenheit


@app(path="/other_app", method="POST", id="other_app")
def other_app(input):

    number = toInt(input)
    temp_times_1_8 = multiply_by_1_8(number)
    return temp_times_1_8


app = start()
