from typing import Any, Dict, List, NamedTuple


class CreatedDatabase(NamedTuple):
    """
    Created database.
    """

    name: str
    username: str
    password: str


def to_created_database(database: Dict[str, Any]) -> CreatedDatabase:
    return CreatedDatabase(
        name=database["database"],
        username=database["username"],
        password=database["password"],
    )


def to_list_databases(databases: Dict[str, Any]) -> List[str]:
    return [database["database"] for database in databases["databases"]]
