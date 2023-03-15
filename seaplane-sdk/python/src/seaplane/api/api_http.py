from typing import Any, Dict, NamedTuple, Optional

import simplejson as json

SDK_HTTP_ERROR_CODE = 0


def headers(api_key: Optional[str]) -> Dict[str, str]:
    return {
        "Accept": "application/json",
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
    }


def to_json(any: NamedTuple) -> Any:
    return json.loads(json.dumps(any), object_hook=_remove_nulls)


def _remove_nulls(d: Dict[Any, Any]) -> Dict[Any, Any]:
    return {k: v for k, v in d.items() if v is not None}
