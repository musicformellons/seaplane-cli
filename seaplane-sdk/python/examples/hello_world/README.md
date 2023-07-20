## Hello World app


Prerequisites:
* Poetry +1.5 (pip install poetry or brew instal poetry)
* Python +1.10

### Create a new project

Install the latest version of seaplane

* `pip install seaplane`

Start a new project

* `seaplane init my_project_name`

This will generate a valid project for you, copy your source files close to the main.py and change the main content as well.

Add your dependencies in your project by poetry add dependency_name or put them in the dependencies section in a toml format.


### Deploy your first Seaplane App


* `cd my_project_name`
* `poetry install`

Don't forget to include your API Key changing it on `main.py`, once you do so you are ready to deploy 🚀:

* `poetry run seaplane deploy`


### How to call Hello World

`main.py`

```python
@task(type="compute", id="hello-world-task")
def hello_world_task(data):
    return "hello world"


@app(id="hello-world-app", path="/hello")
def hello_world_app(data):
    return hello_world_task(data)
```

Call to the POST endpoint:

The URL Path is composed by `/apps/<app-id>/<version>/<app_path>` 

* `app-id` is the `id` defined in `@app` decorator in your pipe. (`"hello-world-app"` in this case)
* `version` right now is going to be always `latest`
* `app_path` is the `path` you defined in `@app` (`"/hello"` in this case)

First you need to have a seaplane TOKEN from an API_KEY

```shell
TOKEN=$(curl -X POST https://flightdeck.cplane.cloud/identity/token --header "Authorization: Bearer ${API_KEY}")
```

POST with two Hello World batch work:

```shell
curl -X POST -H "Content-Type: application/json" -H "Authorization: Bearer ${TOKEN}" -d '{ "input": ["test", "test"] }' https://carrier.staging.cplane.dev/apps/hello-world-app/latest/hello

{"id":"3dc6ec03-6f2f-47f4-9a8e-e289972fb58a","status":"processing"}
```

GET the request result:

* Replace `<request_id>` by the `id` of your POST request (like `3dc6ec03-6f2f-47f4-9a8e-e289972fb58a`)

```shell
curl -X GET -H "Authorization: Bearer ${TOKEN}" https://carrier.staging.cplane.dev/apps/hello-world-app/latest/hello/request/<request_id>
```