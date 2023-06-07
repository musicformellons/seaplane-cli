"""
End to End tests to Global SQL API.

If you are using a new API_KEY first time you run this test will fail.
"""

from seaplane import sea

from . import E2E_API_KEY


def test_list_databases() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    databases = sea.global_sql.list_databases()

    assert len(databases) > 0


def test_create_a_new_database() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    database = sea.global_sql.create_database()

    assert database.name is not None
    assert database.username is not None
    assert database.password is not None


def test_list_new_database_after_creation() -> None:
    sea.config.set_api_key(E2E_API_KEY)

    database_number = len(sea.global_sql.list_databases())
    sea.global_sql.create_database()
    database_num_after_creation = len(sea.global_sql.list_databases())

    assert database_num_after_creation > database_number
