import json
import os
import shutil
from typing import Any, Dict, List, Optional
from urllib.parse import urlparse
import zipfile

import requests

from ..api.api_http import headers
from ..api.api_request import provision_req
from ..api.token_api import TokenAPI
from ..configuration import Configuration, config
from ..logging import log
from ..model.secrets import Secret
from ..util import remove_prefix, unwrap
from .app import App
from .build import PROJECT_TOML, build
from .datasources import tenant_database
from .decorators import context
from .task import Task


def create_task_docker_file(task: Task) -> None:
    docker_file = f"""FROM python:3.10

ENV SEAPLANE_APPS_PRODUCTION True
ENV TASK_ID {task.id}

WORKDIR /app
COPY . .

RUN pip install --no-cache-dir -r requirements.txt
ENTRYPOINT ["python", "demo.py"]
    """

    if not os.path.exists(f"build/{task.id}"):
        os.makedirs(f"build/{task.id}")

    with open(f"build/{task.id}/Dockerfile", "w") as file:
        file.write(docker_file)

    # os.system(
    #    f"docker buildx build --platform linux/arm64,linux/amd64 -t us-central1-docker.pkg.dev/artifacts-356722/demo/tasks/{task.id}:latest  build/{task.id}  --push"  # noqa
    # )


def create_http_api_entry_point_docker_file() -> None:
    docker_file = """FROM python:3.10

ENV SEAPLANE_APPS_PRODUCTION True
ENV PORT 5000

WORKDIR /app
COPY . .

RUN pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn


EXPOSE 5000

CMD gunicorn --bind 0.0.0.0:${PORT} --workers 1 --timeout 300 demo:app
    """

    if not os.path.exists("build/http"):
        os.makedirs("build/http")

    with open("build/http/Dockerfile", "w") as file:
        file.write(docker_file)


def create_subject(app_id: str, task_id: str) -> str:
    return f"{app_id}.{task_id}"


def create_carrier_workload_file(
    tenant: str,
    app_id: str,
    task: Task,
    next_tasks: List[str],
    project_url: str,
) -> Dict[str, Any]:
    output: Optional[Dict[str, Any]] = None

    if len(next_tasks) > 1:
        output = {
            "broker": {
                "outputs": (
                    {"carrier": {"subject": create_subject(app_id, c_id)}} for c_id in next_tasks
                )
            }
        }
    elif len(next_tasks) == 1:
        output = {
            "label": "carrier_out",
            "carrier": {"subject": create_subject(app_id, next_tasks[0])},
        }

    workload = {
        "tenant": tenant,
        "id": task.id,
        "input": {
            "label": "carrier_in",
            "carrier": {
                "subject": create_subject(app_id, task.id),
                "deliver": "all",
                "queue": task.id,
            },
        },
        "processor": {
            "docker": {
                "image": "us-central1-docker.pkg.dev/artifacts-356722/demo/app-executor:latest",  # noqa
                "args": [project_url],
            }
        },
        "output": output,
    }

    if not os.path.exists(f"build/{task.id}"):
        os.makedirs(f"build/{task.id}")

    with open(f"build/{task.id}/workload.json", "w") as file:
        json.dump(workload, file, indent=2)
        log.debug(f"Created {task.id} workload")

    return workload


def copy_project_into_resource(id: str) -> None:
    source_folder = "."
    destination_folder = f"build/{id}"

    if not os.path.exists(f"build/{id}"):
        os.makedirs(f"build/{id}")

    for item in os.listdir(source_folder):
        if os.path.isdir(item) and item == "build":
            continue  # Skip the "build" folder

        elif os.path.isdir(item):
            destination_path = os.path.join(destination_folder, item)
            if os.path.exists(destination_path):
                shutil.rmtree(destination_path)
            shutil.copytree(item, destination_path)
        else:
            destination_path = os.path.join(destination_folder, item)
            shutil.copy2(item, destination_path)


def create_stream(name: str) -> Any:
    log.debug(f"Creating stream: {name}")
    url = f"{config.carrier_endpoint}/stream/{name}"
    req = provision_req(config._token_api)

    payload: Dict[str, str] = {}

    return unwrap(
        req(
            lambda access_token: requests.put(
                url,
                json=payload,
                headers=headers(access_token),
            )
        )
    )


def delete_stream(name: str) -> Any:
    log.debug(f"Deleting stream: {name}")
    url = f"{config.carrier_endpoint}/stream/{name}"
    req = provision_req(config._token_api)

    return unwrap(
        req(
            lambda access_token: requests.delete(
                url,
                headers=headers(access_token),
            )
        )
    )


def get_secrets(config: Configuration) -> List[Secret]:
    secrets = []
    for key, value in config._api_keys.items():
        secrets.append(Secret(key, value))

    return secrets


def add_secrets(name: str, secrets: List[Secret]) -> Any:
    url = f"{config.carrier_endpoint}/flow/{name}/secrets"
    req = provision_req(config._token_api)

    flow_secrets = {}
    for secret in secrets:
        flow_secrets[secret.key] = {"destination": "all", "value": secret.value}

    return unwrap(
        req(
            lambda access_token: requests.put(
                url,
                json=flow_secrets,
                headers=headers(access_token),
            )
        )
    )


def create_flow(name: str, workload: Dict[str, Any]) -> Any:
    log.debug(f"Creating flow: {name}")
    url = f"{config.carrier_endpoint}/flow/{name}"
    req = provision_req(config._token_api)

    return unwrap(
        req(
            lambda access_token: requests.put(
                url,
                json=workload,
                headers=headers(access_token),
            )
        )
    )


def delete_flow(name: str) -> Any:
    log.debug(f"Deleting flow: {name}")

    url = f"{config.carrier_endpoint}/flow/{name}"
    req = provision_req(config._token_api)

    return unwrap(
        req(
            lambda access_token: requests.delete(
                url,
                headers=headers(access_token),
            )
        )
    )


def zip_current_directory(tenant: str, project_name: str) -> str:
    current_directory = os.getcwd()
    zip_filename = f"./build/{tenant}.zip"

    with zipfile.ZipFile(zip_filename, "w", zipfile.ZIP_DEFLATED) as zipf:
        zipf.write(PROJECT_TOML, os.path.relpath(PROJECT_TOML, current_directory))

        for root, _, files in os.walk(f"{current_directory}/{project_name}"):
            for file in files:
                if "__pycache__" in root:
                    continue

                file_path = os.path.join(root, file)
                zipf.write(file_path, os.path.relpath(file_path, current_directory))

    # log.debug(f"Package project for upload: {zip_filename}")
    return zip_filename


def upload_project(project: Dict[str, Any], tenant: str) -> str:
    url = f"https://{urlparse(config.carrier_endpoint).netloc}/apps/upload"
    req = provision_req(config._token_api)

    project_name = project["tool"]["poetry"]["name"]
    project_file = zip_current_directory(tenant, project_name)
    files = {"file": open(project_file, "rb")}

    result: str = unwrap(
        req(
            lambda access_token: requests.post(
                url,
                files=files,
                headers={"Authorization": f"Bearer {access_token}"},
            )
        )
    )

    os.remove(project_file)

    return result


def put_kv(tenant: str, key: str, value: Any) -> Any:
    url = f"https://{urlparse(config.carrier_endpoint).netloc}/apps/kv"
    req = provision_req(config._token_api)

    payload: Dict[str, str] = {"tenant": tenant, "key": key, "value": json.dumps(value)}

    return unwrap(
        req(
            lambda access_token: requests.put(
                url,
                json=payload,
                headers=headers(access_token),
            )
        )
    )


def register_apps_info(tenant: str, schema: Dict[str, Any]) -> None:
    apps = schema["apps"].keys()

    tenant_api_paths: List[Dict[str, str]] = []

    for app_id in apps:
        entry_point_type = schema["apps"][app_id]["entry_point"]["type"]
        if entry_point_type == "API":
            path = schema["apps"][app_id]["entry_point"]["path"]
            method = schema["apps"][app_id]["entry_point"]["method"]
            first_tasks = schema["apps"][app_id]["io"]["entry_point"]

            key = f"{remove_prefix(path, '/')}"
            value = [create_subject(app_id, task_id) for task_id in first_tasks]

            tenant_api_paths.append({"path": path, "method": method})

            put_kv(tenant, key, value)

    put_kv(tenant, "endpoints", tenant_api_paths)


def deploy_task(
    tenant: str,
    app: App,
    task: Task,
    schema: Dict[str, Any],
    secrets: List[Secret],
    project_url: str,
) -> None:
    delete_flow(task.id)

    save_result_task = schema["apps"][app.id]["io"].get("returns", None) == task.id

    save_result_task = schema["apps"][app.id]["io"].get("returns", None) == task.id
    copy_project_into_resource(task.id)
    create_task_docker_file(task)
    next_tasks = schema["apps"][app.id]["io"].get(task.id, None)

    if next_tasks is None:
        next_tasks = []

    workload = create_carrier_workload_file(tenant, app.id, task, next_tasks, project_url)

    save_result_task = schema["apps"][app.id]["io"].get("returns", None) == task.id

    create_flow(task.id, workload)
    secrets.append(Secret("TASK_ID", task.id))
    secrets.append(Secret("SAVE_RESULT_TASK", str(save_result_task)))
    add_secrets(task.id, secrets)

    log.info(f"Deploy for task {task.id} done")


def deploy(task_id: Optional[str] = None) -> None:
    project = build()
    schema = project["schema"]
    tenant = TokenAPI(config).get_tenant()
    tenant_db = tenant_database(tenant)
    secrets = get_secrets(config)
    project_url = upload_project(project["config"], tenant)

    register_apps_info(tenant, schema)

    secrets.append(Secret("SEAPLANE_APPS_PRODUCTION", "true"))
    secrets.append(Secret("SEAPLANE_TENANT_DB__DATABASE", tenant_db.name))
    secrets.append(Secret("SEAPLANE_TENANT_DB_USERNAME", tenant_db.username))
    secrets.append(Secret("SEAPLANE_TENANT_DB_PASSWORD", tenant_db.password))

    if task_id is not None and task_id != "entry_point":
        for sm in context.apps:
            for c in sm.tasks:
                if c.id == task_id:
                    deploy_task(tenant, sm, c, schema, secrets[:], project_url)
    elif task_id is not None and task_id == "entry_point":
        log.info("Deploying entry points...")

        copy_project_into_resource("http")
        create_http_api_entry_point_docker_file()
    else:  # deploy everything
        for sm in context.apps:
            delete_stream(sm.id)
            create_stream(sm.id)

            for c in sm.tasks:
                deploy_task(tenant, sm, c, schema, secrets[:], project_url)

    log.info("Deployment complete")


def destroy() -> None:
    build()

    for sm in context.apps:
        delete_stream(sm.id)

        for c in sm.tasks:
            delete_flow(c.id)
