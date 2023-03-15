from enum import Enum
from typing import List, Optional


class Provider(Enum):
    aws = "AWS"
    azure = "AZURE"
    digital_ocean = "DIGITALOCEAN"
    equinix = "EQUINIX"
    gcp = "GCP"

    def __str__(self) -> str:
        return str(self.value)


def to_providers(providers: Optional[List[str]]) -> List[Provider]:
    if not providers:
        return []

    return [Provider(provider) for provider in providers]
