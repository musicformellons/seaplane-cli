from typing import Any, List, Optional
from urllib.parse import urlparse

import psycopg2
from psycopg2 import OperationalError
from psycopg2.extensions import connection

from ...configuration import config
from ...model.sql import CreatedDatabase


class SqlExecutor:
    def __init__(
        self, database: str, host: str, username: str, password: str, port: int = 5432
    ) -> None:
        self.database = database
        self.host = host
        self.username = username
        self.password = password
        self.port = port

        self.connect()

    @classmethod
    def from_seaplane_database(cls, database: CreatedDatabase) -> Any:
        return cls(
            database=database.name,
            host=urlparse(config.global_sql_endpoint).netloc,
            username=database.username,
            password=database.password,
            port=5432,
        )

    def connect(self) -> None:
        self.conn: connection = psycopg2.connect(
            database=self.database,
            host=self.host,
            user=self.username,
            password=self.password,
            port=self.port,
        )

        self.conn.set_session(autocommit=True)

    def check_connection(self) -> None:
        try:
            cursor = self.conn.cursor()
            cursor.execute("SELECT current_database()")
            cursor.close()
        except OperationalError:
            self.connect()

    def execute(self, sql: str, parameters: Optional[List[Any]] = None) -> int:
        self.check_connection()

        cursor = self.conn.cursor()
        cursor.execute(sql, parameters)
        row_count: int = cursor.rowcount
        cursor.close()
        return row_count

    def insert(self, sql: str, parameters: Optional[List[Any]] = None) -> int:
        return self.execute(sql, parameters)

    def fetch_one(self, sql: str, parameters: Optional[List[Any]] = None) -> Any:
        self.check_connection()

        cursor = self.conn.cursor()
        cursor.execute(sql, parameters)
        result = cursor.fetchone()
        cursor.close()
        return result

    def fetch_all(self, sql: str, parameters: Optional[List[Any]] = None) -> Any:
        self.check_connection()

        cursor = self.conn.cursor()
        cursor.execute(sql, parameters)
        result = cursor.fetchall()
        cursor.close()
        return result
