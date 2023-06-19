from typing import Any, Callable, Dict, List, Optional, Tuple

import psycopg2
from psycopg2.extensions import connection

from ...configuration import config
from ...logging import log


class SqlExecutor:
    def __init__(self, conn: connection) -> None:
        self.conn = conn

    def insert(self, sql: str, parameters: Optional[List[Any]] = None) -> int:
        cursor = self.conn.cursor()
        cursor.execute(sql, parameters)
        row_count: int = cursor.rowcount
        cursor.close()
        return row_count

    def query(self, sql: str) -> Any:
        cursor = self.conn.cursor()
        cursor.execute(sql)
        result = cursor.fetchall()
        cursor.close()
        return result


class Sql:
    def __init__(self, func: Callable[[Any], Any], id: str, sql: Dict[str, str]) -> None:
        self.func = func
        self.args: Optional[Tuple[Any, ...]] = None
        self.kwargs: Optional[Dict[str, Any]] = None
        self.type = "sql"
        self.id = id
        self.sql = sql

        username = self.sql["username"]
        password = self.sql["password"]
        database = self.sql["database"]

        self.conn: connection = psycopg2.connect(
            database=database,
            host=config.global_sql_endpoint,
            user=username,
            password=password,
            port=5432,
        )

        self.conn.set_session(autocommit=True)

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        log.info("Processing SQL Coprocessor...")

        self.args = self.args + (SqlExecutor(self.conn),)

        return self.func(*self.args, **self.kwargs)
