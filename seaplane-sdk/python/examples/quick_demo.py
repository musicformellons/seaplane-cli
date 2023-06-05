from seaplane import config, coprocessor, smartpipe, start

api_keys = {
    "SEA_API_KEY": "...",  # Seaplane Coprocessors
}

config.set_api_keys(api_keys)

@coprocessor(type="inference", model="bloom", id="my-bloom-coprocessor")
def bloom_inference(input, model):

        # run your inference here
    return model(input)

@smartpipe(path="/my-api-endpoint", method="POST", id="my-smart-pipe")
def my_smartpipe(body):    

    return bloom_inference(body)


start()
