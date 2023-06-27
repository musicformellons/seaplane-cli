from typing import Any, Callable, Dict, Optional, Tuple
from urllib.parse import urlparse

from ...configuration import config
from ...logging import log
from ..datasources import SqlExecutor


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
        hostname = self.sql.get("hostname", urlparse(config.global_sql_endpoint).netloc)
        port = self.sql.get("port", 5432)

        self.executor = SqlExecutor(database, hostname, username, password, int(port))

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        log.info("Processing SQL Task...")

        self.args = self.args + (self.executor,)

        return self.func(*self.args, **self.kwargs)
