export const tutorial = `
#### 📭 You don't have any App yet, follow this tutorial to create your first App.

## Seaplane Apps

### Installation

~~~shell
pip install seaplane
~~~

### Configure your API KEYS

For using some of the available Tasks, you have to provide some of the API KEYS. 


~~~python
from seaplane import sea

api_keys = {
    "SEAPLANE_API_KEY": "...",  # Seaplane Tasks
    "OPENAI_API_KEY": "...", # OpenAI Task
    "REPLICATE_API_KEY": "...",  # Replicate Task
}

config.set_api_keys(api_keys)
~~~

or If you only need to set up the Seaplane API Key, you can use ***config.set_api_key*** :

~~~python
config.set_api_key("...")
~~~

### Usage

For writing your first App you have to import four elements from the Seaplane Python SDK, ***config***, ***app***, ***task*** and ***start***

* ***config*** is the Configuration Object for setting the API Keys
* ***app*** is the decorator for defining a Seaplane App
* ***task*** is the decorator for defining a Seaplane Task
* ***start*** is the function needed to run your Apps locally, It needs to locale it at the end of the Apps file.

You can run this App locally if you have a Seaplane API Key:

demo.py:

~~~python
from seaplane import config, app, task, start

api_keys = {
    "SEAPLANE_API_KEY": "sp-test-api-key",  # Seaplane Tasks
}

config.set_api_keys(api_keys)

@app(path='/my-api-endpoint', method="POST", id='my-app')
def my_app(body):
  
    @task(type='inference', model='bloom', id='my-bloom-task')
    def bloom_inference(input, model):

      # run your inference here
      return model(input)
  
    return bloom_inference(body)

start()
~~~

⚠️ Don't forget **start()** at the end of the file.

~~~shell
$ python demo.py
$[Seaplane] 🧠 App: my-app, Path: /my-api-endpoint
$ * 
$ * Running on http://127.0.0.1:1337
~~~

You'll able to call ***my-app*** with the following curl:

~~~curl
curl -X POST -H "Content-Type: application/json" -d 'This is a test' http://127.0.0.1:1337/my-api-endpoint
~~~

## Available LLM Models

* Seaplane Bloom ID: ***model='bloom'***
* OpenAI GPT-3 ID: ***model='GPT-3'***
* OpenAI GPT-3.5 ID: ***model='GPT-3.5'***
* Replicate Stable Diffusion 1.5 ID: ***model='stable-diffusion'***

For using this models you have to indicate in the task of ***type='inference'*** which model you want to use for example **bloom** using ***model='bloom'***


`