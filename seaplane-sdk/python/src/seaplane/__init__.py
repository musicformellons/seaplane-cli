from importlib.metadata import version

from .api.compute_api import ComputeAPI
from .api.lock_api import LockAPI
from .api.metadata_api import MetadataAPI
from .api.restrict_api import RestrictAPI
from .api.sql_api import GlobalSQL
from .api.token_api import TokenAPI
from .apps import App, Task, app, build, context, start, task
from .configuration import Configuration, config
from .logging import log

__version__ = version(__name__)


class Seaplane:
    @property
    def config(self) -> Configuration:
        return config

    @property
    def auth(self) -> TokenAPI:
        return config._token_api

    @property
    def metadata(self) -> MetadataAPI:
        return MetadataAPI(config)

    @property
    def locks(self) -> LockAPI:
        return LockAPI(config)

    @property
    def restrict(self) -> RestrictAPI:
        return RestrictAPI(config)

    @property
    def compute(self) -> ComputeAPI:
        return ComputeAPI(config)

    @property
    def global_sql(self) -> GlobalSQL:
        return GlobalSQL(config)


sea = Seaplane()

__all__ = [
    "App",
    "Task",
    "task",
    "app",
    "start",
    "log",
    "context",
    "build",
]
