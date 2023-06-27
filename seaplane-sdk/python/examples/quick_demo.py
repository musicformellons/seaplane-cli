from seaplane import app, config, start, task

api_keys = {
    "SEAPLANE_API_KEY": "...",  # Seaplane Tasks
}

config.set_api_keys(api_keys)


@task(type="inference", model="bloom", id="my-bloom-task")
def bloom_inference(input, model):

    # run your inference here
    return model(input)


@app(path="/my-api-endpoint", method="POST", id="my-app")
def my_app(body):

    return bloom_inference(body)


start()
