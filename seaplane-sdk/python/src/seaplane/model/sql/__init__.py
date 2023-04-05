from typing import Any, Dict, NamedTuple


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
