from seaplane import config, coprocessor, log, smartpipe, start
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)
config.set_global_sql_endpoint("sql.staging.cplane.dev")

sql_access = {"username": "...", "password": "...", "database": "...", "port": 2001}


@coprocessor(type="sql", sql=sql_access)
def query(sql):

    return sql.fetch_all(""" SELECT * FROM joe_results_draft""")


@smartpipe(path="/query_database", id="query_db")
def query_database(input):

    return query()


app = start()
