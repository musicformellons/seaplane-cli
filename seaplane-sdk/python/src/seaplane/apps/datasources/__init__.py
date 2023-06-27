import json
import os

from ...api.sql_api import GlobalSQL
from ...configuration import config
from ...logging import log
from ...model.sql import CreatedDatabase
from .request_data_source import RequestDataSource
from .sql_executor import SqlExecutor

requests_table = """
CREATE TABLE requests (
     id VARCHAR PRIMARY KEY,
     batch_count INTEGER
);
"""

results_table = """
CREATE TABLE results (
    id SERIAL PRIMARY KEY,
    request_id VARCHAR UNIQUE,
    original_order INTEGER,
    output JSONB
);

ALTER SEQUENCE results_id_seq START WITH 10 INCREMENT BY 1;
CREATE INDEX idx_request_id ON results(request_id);
"""


def create_schema(database: CreatedDatabase) -> None:
    attemps = 0
    log.debug("Creating db schemas...")

    while attemps < 3:
        try:
            sql = SqlExecutor.from_seaplane_database(database)

            sql.execute(requests_table)
            sql.execute(results_table)
            attemps = 3
        except Exception:
            attemps = attemps + 1
            log.debug(f"Create schema error {attemps}")


def tenant_database() -> CreatedDatabase:
    if os.path.exists(".seaplane"):
        log.debug("Tenant db provisioning exists, using .seaplane config")

        with open(".seaplane", "r") as file:
            database = json.load(file)
            return CreatedDatabase(**database)
    else:
        log.debug("Tenant db provisioning doesn't exist, creating .seaplane config")
        sql = GlobalSQL(config)
        new_database = sql.create_database()

        with open(".seaplane", "w") as file:
            db_info = json.dumps(new_database._asdict())
            file.write(db_info)

        create_schema(new_database)

        return new_database


__all__ = ["RequestDataSource", "SqlExecutor"]
