# Seaplane Smart Pipes Python SDK
[![PyPI](https://badge.fury.io/py/seaplane.svg)](https://badge.fury.io/py/seaplane)
[![Python](https://img.shields.io/pypi/pyversions/seaplane.svg?style=plastic)](https://badge.fury.io/py/seaplane)

Simple Python library to Develop, Test and Ship LLM applications fast.

## What is Seaplane?

Seaplane is the global platform for building and scaling your AI application stack
without the complexity of managing cloud infrastructure.

It serves as a reference application for how our APIs can be utilized.

## What are Smart Pipes?

Smart pipes form the basis for any machine-learning workflow. 

They define a data stream on which coprocessors and models can be run. They can be defined programmatically using the @smartpipe decorator or through the GUI in the drag-and-drop interface. 

Smart pipes automatically set up the infrastructure required for the data stream and coprocessors based on the user-defined configuration_settings. Seaplane automatically scales the underlying infrastructure based on the throughput of the Smart Pipe. 

```python
from seaplane import smartpipe

@smartpipe(path='/my-api-endpoint', method="POST", id='my-smart-pipe')
def my_smartpipe(body):  
  ...
	# your models and coprocessors go here
```

## Coprocessors

Coprocessors that run inside a Smart Pipe can contain any kind of code. Model coprocessors are specifically designed to run inference models and use a smart combination of CPU, GPU, and Memory to reduce inferencing costs. A coprocessor is defined through the @coprocessor decorator. 

You can use external models as well throught Coprocessors to integrate it in your Smart Pipe, like OpenAI models and Open Source Stable Diffusion text-to-image LLMs.

we can define an **inference** model to run inside a Smart Pipes as follows. We mark the coprocessor as `type='inference'` to take advantage of the inference specific hardware (this allows us to reduce inferencing costs). We specify the model id. This model should match a model that you previously uploaded to the Seaplane Model Registry. The model is available through the model parameter object in your function. 

```python
from seaplane import smartpipe, coprocessor

@coprocessor(type='inference', model='my-model-id', id='my-coprocessor')
def my_inference_model(input, model):

    # run your inference here
    return model(input)

@smartpipe(path='/my-api-endpoint', method="POST", id='my-smart-pipe')
def my_smartpipe(body):
	    
    return my_inference_model(body) 
```

Besides inference models, coprocessors can run any function you define or any containerized workload. For example, you can run the following coprocessor to transform the input from `string` into an `int`. The `type=compute` signals that this is a normal coprocessor and does not require inference-specific hardware. 

```python
from seaplane import smartpipe, coprocessor

@coprocessor(type='compute', id='my-coprocessor')
def my_function(input_data):
		
		# convert string to int and sum two
    return int(input_data) + 2

@smartpipe(path='/my-api-endpoint', method="POST", id='my-smart-pipe')
def my_smartpipe(body):    
    
    return my_function(body)
```

You can chain multiple coprocessors together to get the desired result. Think of this as your machine-learning pipeline. For example, the Smart Pipe below creates an API endpoint that takes a `string` as input, converts it to an `int`, and runs it through an inference model. Finally, we format the inference result and return it to the user that requested it. 

Each individual coprocessor in the pipeline can scale individually based on the throughput.

```python
from seaplane import smartpipe, coprocessor

# convert string to int 
@coprocessor(type='compute', id='convert-int')
def convert_string(input_data):
		
		return int(input_data)

# run inference
@coprocessor(type='inference', model='my-model', id='my-inference')
def inference(number, model):
		
		return model(number)

# format the result
@coprocessor(type='compute', id='format-result')
def format_result(inferenced_result, input_data):
		
	result = {
	"input" : input_data, 
	"inferenced_result" : inferenced_result		
	}

	return result

@smartpipe(path='/inference_number', method='post', id='my-smart-pipe')
def my_smartpipe(body):	

	converted_data = convert_string(body)
	inferenced_result = inference(converted_data)
	return format_result(inferenced_result, body)  
```

## Available LLM Models

* Seaplane Bloom ID: `bloom`
* OpenAI GPT-3 ID: `GPT-3`
* OpenAI GPT-3.5 ID: `GPT-3.5`
* Replicate Stable Diffusion 1.5 ID: `stable-diffusion`

For using this models you have to indicate in the coprocessor of `type='inference'` which model you want to use for example **bloom** using `model='bloom'` :


```python
from seaplane import smartpipe, coprocessor

@coprocessor(type='inference', model='bloom', id='my-bloom-coprocessor')
def bloom_inference(input, model):

		# run your inference here
		return model(input)

@smartpipe(path='/my-api-endpoint', method="POST", id='my-smart-pipe')
def my_smartpipe(body):		      

    return bloom_inference(body)
```

## Installation

```shell
pip install seaplane
```

## Configure your API KEYS

For using some of the available Coprocessors, you have to provide some of the API KEYS. 


```python
from seaplane import sea

api_keys = {
    "SEA_API_KEY": "...",  # Seaplane Coprocessors
    "OPENAI_API_KEY": "...", # OpenAI Coprocessor
    "RE_API_KEY": "...",  # Replicate Coprocessor
}

config.set_api_keys(api_keys)
```

or If you only need to set up the Seaplane API Key, you can use `config.set_api_key` :

```python
config.set_api_key("...")
```

## Usage

For writing your first Smart Pipe you have to import four elements from the Seaplane Python SDK, `config`, `smartpipe`, `coprocessor` and `start`

* `config` is the Configuration Object for setting the API Keys
* `smartpipe` is the decorator for defining a Seaplane Smart Pipe
* `coprocessor` is the decorator for defining a Seaplane Coprocessor
* `start` is the function needed to run your Smart Pipes locally, It needs to locale it at the end of the Smart Pipes file.

You can run this Smart Pipe locally if you have a Seaplane API Key:

demo.py:

```python
from seaplane import config, smartpipe, coprocessor, start

api_keys = {
    "SEA_API_KEY": "sp-test-api-key",  # Seaplane Coprocessors
}

config.set_api_keys(api_keys)

@coprocessor(type='inference', model='bloom', id='my-bloom-coprocessor')
def bloom_inference(input, model):

  	# run your inference here
  	return model(input)

@smartpipe(path='/my-api-endpoint', method="POST", id='my-smart-pipe')
def my_smartpipe(body):
      
    return bloom_inference(body)

start()
```

⚠️ Don't forget **start()** at the end of the file.

```shell
$ python demo.py
$[Seaplane] 🧠 Smart Pipe: my-smart-pipe, Path: /my-api-endpoint
$ * Serving Flask app 'seaplane.smartpipes.smartapi'
$ * Debug mode: off
$ * Running on http://127.0.0.1:1337
```

You'll able to call `my-smart-pipe` with the following curl:

```curl
curl -X POST -H "Content-Type: application/json" -d 'This is a test' http://127.0.0.1:1337/my-api-endpoint
```

## Interactive Smart Pipes Website

![Screenshot 2023-05-10 at 19 07 42](https://github.com/seaplane-io/seaplane/assets/5845622/c346c22b-4fca-4062-8207-38f61f315857)

You can download the website and run it on local using the following script, `download.sh`. You need to have **npm** installed in your machine.

```shell
#!/bin/bash

URL="https://github.com/seaplane-io/seaplane/releases/download/sdk-py-v0.2.7/smartpipes-0.2.9.tar.gz"
curl -LO "$URL"

mkdir website
tar -xzf smartpipes-0.2.9.tar.gz -C website --strip-components=1

cd website || exit
npm install
npm run dev
```

Give execution permissions and execute it: 

```shell
$ chmod 700 download.sh
$ ./download.sh
```

It'll download the website, install the dependencies and run it locally at `http://localhost:3000` . It is a NextJS website so you can run again using **npm**:

```shell
$ npm run dev
```

Open `https://localhost:3000` in any browser, inside of each Smart Pipe you can execute them without `curl` command and see the Smart Pipe requests in real time.

![screencapture-localhost-3000-2023-05-10-19_20_08](https://github.com/seaplane-io/seaplane/assets/5845622/f08db868-834f-4e30-9535-65090a39e374)


## License

Licensed under the Apache License, Version 2.0, [LICENSE]. Copyright 2022 Seaplane IO, Inc.

[//]: # (Links)

[Seaplane]: https://seaplane.io/
[CLI]: https://github.com/seaplane-io/seaplane/tree/main/seaplane-cli
[SDK]: https://github.com/seaplane-io/seaplane/tree/main/seaplane
[Getting Started]: https://github.com/seaplane-io/seaplane/blob/main/seaplane-sdk/python/docs/quickstart.md
[CONTRIBUTING]: https://github.com/seaplane-io/seaplane/tree/main/seaplane-sdk/python/CONTRIBUTIONS.md
[LICENSE]: https://github.com/seaplane-io/seaplane/blob/main/LICENSE
