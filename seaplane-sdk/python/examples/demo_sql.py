from seaplane import coprocessor, log, smartpipe, start
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)

sql_access = {
    "username": "...",
    "password": "...",
    "database": "...",
}


@coprocessor(type="sql", sql=sql_access)
def query(sql):

    return sql.query(""" SELECT * FROM joe_results_draft""")


@smartpipe(path="/query_database", id="query_db")
def query_database(input):

    return query()


app = start()
