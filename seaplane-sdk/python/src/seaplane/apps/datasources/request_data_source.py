from typing import Any, Dict, List, Optional

from psycopg2.extras import Json

from .sql_executor import SqlExecutor


class RequestDataSource:
    def __init__(
        self, database: str, host: str, username: str, password: str, port: int = 5432
    ) -> None:
        self.executor = SqlExecutor(database, host, username, password, port)

    def save_request(self, id: str, batch_count: int) -> int:
        return self.executor.insert(
            """ INSERT INTO requests
                        (id, batch_count)
                        VALUES
                        (%s, %s)""",
            [id, batch_count],
        )

    def save_result(self, request_id: str, order: int, output: Any) -> int:
        return self.executor.insert(
            """ INSERT INTO results
                        (request_id, original_order, output)
                        VALUES
                        (%s, %s, %s)""",
            [request_id, order, Json(output)],
        )

    def get_request_status(self, request_id: str) -> Optional[Any]:
        row = self.executor.fetch_one(
            """
          SELECT r.id, r.batch_count,
              (SELECT COUNT(*) FROM results rs WHERE rs.request_id = r.id) AS result_count
          FROM requests r
          WHERE r.id = %s;
        """,
            [request_id],
        )
        result = None
        if row:
            result = {"request_id": row[0], "batch_count": row[1], "result_count": row[2]}
        return result

    def get_request_results(self, request_id: str) -> List[Dict[str, Any]]:
        rows = self.executor.fetch_all(
            """
          SELECT output
          FROM results
          WHERE request_id = %s
          ORDER BY original_order ASC;
        """,
            [request_id],
        )

        results = []
        for row in rows:
            results.append(row[0])

        return results
