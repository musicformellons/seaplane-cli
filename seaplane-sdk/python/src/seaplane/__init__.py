from .api.lock_api import LockAPI
from .api.metadata_api import MetadataAPI
from .api.restrict_api import RestrictAPI
from .api.compute_api import ComputeAPI
from .api.token_api import TokenAPI
from .configuration import Configuration, config


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

sea = Seaplane()
