## Seaplane Apps project

## What are Apps?

Apps form the basis for any machine-learning workflow. 

They define a data stream on which tasks and models can be run. They can be defined programmatically using the @app decorator or through the GUI in the drag-and-drop interface. 

Apps automatically set up the infrastructure required for the data stream and tasks based on the user-defined configuration_settings. Seaplane automatically scales the underlying infrastructure based on the throughput of the App. 

```python
from seaplane import app

@app(path='/my-api-endpoint', id='my-app')
def my_app(body):  
  ...
	# your models and tasks go here
```

## Tasks

Tasks that run inside a App can contain any kind of code. Model tasks are specifically designed to run inference models and use a smart combination of CPU, GPU, and Memory to reduce inferencing costs. A task is defined through the @task decorator. 

You can use external models as well through Tasks to integrate it in your App, like OpenAI models and Open Source Stable Diffusion text-to-image LLMs.

we can define an **inference** model to run inside a Apps as follows. We mark the task as `type='inference'` to take advantage of the inference specific hardware (this allows us to reduce inferencing costs). We specify the model id. This model should match a model that you previously uploaded to the Seaplane Model Registry. The model is available through the model parameter object in your function. 

```python
from seaplane import app, task

@task(type='inference', model='my-model-id', id='my-task')
def my_inference_model(input, model):

    # run your inference here
    return model(input)

@app(path='/my-api-endpoint', id='my-app')
def my_app(body):
	    
    return my_inference_model(body) 
```

Besides inference models, tasks can run any function you define or any containerized workload. For example, you can run the following task to transform the input from `string` into an `int`. The `type=compute` signals that this is a normal task and does not require inference-specific hardware. 

```python
from seaplane import app, task

@task(type='compute', id='my-task')
def my_function(input_data):
		
		# convert string to int and sum two
    return int(input_data) + 2

@app(path='/my-api-endpoint', id='my-app')
def my_app(body):    
    
    return my_function(body)
```

You can chain multiple tasks together to get the desired result. Think of this as your machine-learning pipeline. For example, the App below creates an API endpoint that takes a `string` as input, converts it to an `int`, and runs it through an inference model. Finally, we format the inference result and return it to the user that requested it. 

Each individual task in the pipeline can scale individually based on the throughput.

```python
from seaplane import app, task

# convert string to int 
@task(type='compute', id='convert-int')
def convert_string(input_data):
		
		return int(input_data)

# run inference
@task(type='inference', model='my-model', id='my-inference')
def inference(number, model):
		
		return model(number)

# format the result
@task(type='compute', id='format-result')
def format_result(inferenced_result, input_data):
		
	result = {
	"input" : input_data, 
	"inferenced_result" : inferenced_result		
	}

	return result

@app(path='/inference_number', method='post', id='my-app')
def my_app(body):	

	converted_data = convert_string(body)
	inferenced_result = inference(converted_data)
	return format_result(inferenced_result, body)  
```

## Installation

Prerequisites:

* Python +3.10
* Poetry


```shell
poetry install
```

## Project Structure

As any other Poetry project you need to follow a simple structure where your source files are under the project name folder.

my_project_name
├── README.md
├── my_project_name
│   └── main.py
├── pyproject.toml
└── tests
    └── __init__.py

## Configure your API KEYS

For using some of the available Tasks, you have to provide some of the API KEYS. 


```python
from seaplane import sea

api_keys = {
    "SEAPLANE_API_KEY": "...",  # Seaplane Tasks
    "OPENAI_API_KEY": "...", # OpenAI Task
}

config.set_api_keys(api_keys)
```

or If you only need to set up the Seaplane API Key, you can use `config.set_api_key` :

```python
config.set_api_key("...")
```

additionally, if you don't want to use API Keys in your code you can provide them using ENV variables like:

- SEAPLANE_API_KEY
- OPENAI_API_KEY

## Usage

For writing your first App you have to import four elements from the Seaplane Python SDK, `config`, `app`, `task` and `start`

* `config` is the Configuration Object for setting the API Keys
* `app` is the decorator for defining a Seaplane App
* `task` is the decorator for defining a Seaplane Task
* `start` is the function needed to run your Apps, It needs to locale it at the end of the Apps file.

You can run this App locally if you have a Seaplane API Key:

demo.py:

```python
from seaplane import config, app, task, start

api_keys = {
    "SEAPLANE_API_KEY": "sp-test-api-key",  # Seaplane Tasks
}

config.set_api_keys(api_keys)

@task(type='inference', model='bloom', id='my-bloom-task')
def bloom_inference(input, model):

  	# run your inference here
  	return model(input)

@app(path='/my-api-endpoint', id='my-app')
def my_app(body):
      
    return bloom_inference(body)

start()
```

⚠️ Don't forget **start()** at the end of the file.


## Deploy and production usage

Deploy: `poetry run python3 seaplane deploy`

This will deploy your pipelines into Seaplane making them accessible throught the App's request.

### Your first request to an App

Apps endpoint are the entry point to your pipeline, which has to follow the next schema:

```json
{
	"input": [ any_object ],
	"params": {} //this params will be shared between each any_object calling to the first task
}
```

When you define your HTTP `@app` you have to include the `/path` you want to use, that path is used in the App's request.

```
curl -X POST -H "Content-Type: application/json" -H "Authorization: Bearer ${TOKEN}" -d '{ "input": [{ ... }], "params": { ... } }' https://carrier.staging.cplane.dev/apps/request/<app_path>

{"id":"3dc6ec03-6f2f-47f4-9a8e-e289972fb58a","status":"processing"}
```

App's request response will returns an Request ID, you have to use it in the App's request GET endpoint.

```
curl -H "Authorization: Bearer ${TOKEN}" https://carrier.staging.cplane.dev/apps/request/<request_id>

{"id": "3dc6ec03-6f2f-47f4-9a8e-e289972fb58a", "output": [ .. output ordered ..], "status": "completed"}
```

You can know which endpoints you do have available using `GET /endpoints` or looking into your project the `@app` paths.

```
curl -X GET -H "Authorization: Bearer ${TOKEN}" https://carrier.staging.cplane.dev/apps/endpoints
```


## License

Licensed under the Apache License, Version 2.0, [LICENSE]. Copyright 2022 Seaplane IO, Inc.

[//]: # (Links)

[Seaplane]: https://seaplane.io/
[CLI]: https://github.com/seaplane-io/seaplane/tree/main/seaplane-cli
[SDK]: https://github.com/seaplane-io/seaplane/tree/main/seaplane
[Getting Started]: https://github.com/seaplane-io/seaplane/blob/main/seaplane-sdk/python/docs/quickstart.md
[CONTRIBUTING]: https://github.com/seaplane-io/seaplane/tree/main/seaplane-sdk/python/CONTRIBUTIONS.md
[LICENSE]: https://github.com/seaplane-io/seaplane/blob/main/LICENSE
