from seaplane import app, config, log, start, task
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)

sql_access = {"username": "...", "password": "...", "database": "...", "port": 2001}


@task(type="sql", sql=sql_access)
def insert(input, sql):

    return sql.insert(""" INSERT INTO my_table VALUES (%s, %s)""", input.a, input.b)


@app(path="/insert", id="query_db")
def insert(input):

    return insert(input)


app = start()
