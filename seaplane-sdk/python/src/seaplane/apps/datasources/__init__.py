import json
from typing import Any, Dict
from urllib.parse import urlparse

import requests

from ...api.api_http import headers
from ...api.api_request import provision_req
from ...api.sql_api import GlobalSQL
from ...configuration import config
from ...logging import log
from ...model.sql import CreatedDatabase
from ...util import unwrap
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
    request_id VARCHAR,
    original_order INTEGER,
    output JSONB
);

ALTER SEQUENCE results_id_seq START WITH 10 INCREMENT BY 1;
CREATE INDEX idx_request_id ON results(request_id);
"""


def create_schema(database: CreatedDatabase) -> None:
    attempts = 0
    exit = False
    log.debug("Creating db schemas...")

    while attempts < 3 and not exit:
        try:
            sql = SqlExecutor.from_seaplane_database(database)

            sql.execute(requests_table)
            sql.execute(results_table)
            exit = True
        except Exception as e:
            log.error(f"Create schema error: {e}")
            attempts = attempts + 1
            log.error(f"attempt: {attempts}")

    if attempts == 3:
        log.debug("Error creating the default DB tables")


def get_default_db_info(tenant: str) -> Any:
    url = f"https://{urlparse(config.carrier_endpoint).netloc}/apps/kv/{tenant}/default_db"
    req = provision_req(config._token_api)

    return unwrap(
        req(
            lambda access_token: requests.get(
                url,
                headers=headers(access_token),
            )
        )
    )


def put_database(tenant: str, created_database: CreatedDatabase) -> Any:
    url = f"https://{urlparse(config.carrier_endpoint).netloc}/apps/kv"
    req = provision_req(config._token_api)

    payload: Dict[str, str] = {
        "tenant": tenant,
        "key": "default_db",
        "value": json.dumps(created_database._asdict()),
    }

    return unwrap(
        req(
            lambda access_token: requests.put(
                url,
                json=payload,
                headers=headers(access_token),
            )
        )
    )


def tenant_database(tenant: str) -> CreatedDatabase:
    default_db = get_default_db_info(tenant)

    if not default_db:
        log.debug("Default DB doesn't exist, creating DB...")
        sql = GlobalSQL(config)
        new_database = sql.create_database()

        databases = sql.list_databases()

        while new_database.name not in databases:
            databases = sql.list_databases()

        create_schema(new_database)
        put_database(tenant, new_database)

        return new_database
    else:
        return CreatedDatabase(**default_db)


__all__ = ["RequestDataSource", "SqlExecutor"]
