from seaplane import Coprocessor, config, coprocessor, log, smartpipe, start
from seaplane.logging import SeaLogger

SEAPLANE_API_KEY = "..."
OPENAI_API_KEY = "..."
REPLICATE_API_KEY = "..."

log.level(SeaLogger.DEBUG)

api_keys = {
    "SEAPLANE_API_KEY": SEAPLANE_API_KEY,
    "OPENAI_API_KEY": OPENAI_API_KEY,
    "REPLICATE_API_KEY": REPLICATE_API_KEY,
}
config.set_api_keys(api_keys)

stable_diffusion_prompt_contenxt = """
I want you to help me make requests (prompts) for the Stable Diffusion neural network.

Stable diffusion is a text-based image generation model that can create diverse and high-quality images based on your requests. In order to get the best results from Stable diffusion, you need to follow some guidelines when composing prompts.

Here are some tips for writing prompts for Stable diffusion1:

1) Be as specific as possible in your requests. Stable diffusion handles concrete prompts better than abstract or ambiguous ones. For example, instead of “portrait of a woman” it is better to write “portrait of a woman with brown eyes and red hair in Renaissance style”.
2) Specify specific art styles or materials. If you want to get an image in a certain style or with a certain texture, then specify this in your request. For example, instead of “landscape” it is better to write “watercolor landscape with mountains and lake".
3) Specify specific artists for reference. If you want to get an image similar to the work of some artist, then specify his name in your request. For example, instead of “abstract image” it is better to write “abstract image in the style of Picasso”.
4) Weigh your keywords. You can use token:1.3 to specify the weight of keywords in your query. The greater the weight of the keyword, the more it will affect the result. For example, if you want to get an image of a cat with green eyes and a pink nose, then you can write “a cat:1.5, green eyes:1.3,pink nose:1”. This means that the cat will be the most important element of the image, the green eyes will be less important, and the pink nose will be the least important.
Another way to adjust the strength of a keyword is to use () and []. (keyword) increases the strength of the keyword by 1.1 times and is equivalent to (keyword:1.1). [keyword] reduces the strength of the keyword by 0.9 times and corresponds to (keyword:0.9).

You can use several of them, as in algebra... The effect is multiplicative.

(keyword): 1.1
((keyword)): 1.21
(((keyword))): 1.33


Similarly, the effects of using multiple [] are as follows

[keyword]: 0.9
[[keyword]]: 0.81
[[[keyword]]]: 0.73

I will also give some examples of good prompts for this neural network so that you can study them and focus on them.



Examples:

a cute kitten made out of metal, (cyborg:1.1), ([tail | detailed wire]:1.3), (intricate details), hdr, (intricate details, hyperdetailed:1.2), cinematic shot, vignette, centered

medical mask, victorian era, cinematography, intricately detailed, crafted, meticulous, magnificent, maximum details, extremely hyper aesthetic

a girl, wearing a tie, cupcake in her hands, school, indoors, (soothing tones:1.25), (hdr:1.25), (artstation:1.2), dramatic, (intricate details:1.14), (hyperrealistic 3d render:1.16), (filmic:0.55), (rutkowski:1.1), (faded:1.3)

Jane Eyre with headphones, natural skin texture, 24mm, 4k textures, soft cinematic light, adobe lightroom, photolab, hdr, intricate, elegant, highly detailed, sharp focus, ((((cinematic look)))), soothing tones, insane details, intricate details, hyperdetailed, low contrast, soft cinematic light, dim colors, exposure blend, hdr, faded

a portrait of a laughing, toxic, muscle, god, elder, (hdr:1.28), bald, hyperdetailed, cinematic, warm lights, intricate details, hyperrealistic, dark radial background, (muted colors:1.38), (neutral colors:1.2)

My query may be in other languages. In that case, translate it into English. Your answer is exclusively in English (IMPORTANT!!!), since the model only understands it.
Also, you should not copy my request directly in your response, you should compose a new one, observing the format given in the examples.
Don't add your comments, but answer right away.

My first request is the following, please avoid showing screens, devices, or objects, show landscapes, backgrounds, beatufiul images: 
"""


@coprocessor(type="compute", id="string_to_int")
def toInt(input):

    return int(input)


@coprocessor(type="compute", id="multiply_by_1_8")
def multiply_by_1_8(number):

    return number * 1.8


@coprocessor(type="compute", id="add_32")
def add_32(number):

    return number + 32


@smartpipe(path="/celsius_to_fahrenheit", method="POST", id="celsius_to_fahrenheit")
def celsius_to_fahrenheit(input):

    number = toInt(input)
    temp_times_1_8 = multiply_by_1_8(number)
    temp_fahrenheit = add_32(temp_times_1_8)
    return temp_fahrenheit


@coprocessor(type="compute", id="convert-int")
def convert_string(data):

    return int(data)


@coprocessor(type="compute", id="multiply-by-two")
def double_a_number(data):

    return data * 2


@smartpipe(path="/inference_number", method="POST", id="my-smart-pipe")
def my_smartpipe(input_data=None):

    seed = double_a_number(convert_string(input_data))

    return seed


@coprocessor(type="inference", model="gpt-3.5", id="create_a_blog_post")
def create_a_blog_post(topic, model):
    prompt = f"Create a blog post of this topic: {topic}, not so long"

    payload = {
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.7,
    }
    result = model(payload)

    return result["choices"][0]["message"]["content"]


@coprocessor(type="inference", model="gpt-3.5", id="get_prompt")
def get_prompt(blog_post, model):
    prompt = f"{stable_diffusion_prompt_contenxt} \n {blog_post}"

    payload = {
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.7,
    }

    result = model(payload)

    return result["choices"][0]["message"]["content"]


@coprocessor(type="inference", model="stable-diffusion", id="text-to-image")
def get_image_from_prompt(prompt, model):

    result = model(prompt)
    return result["output"]


@smartpipe(path="/create_blog_post_with_images", method="POST", id="write-blog-post")
def blog_post_to_image(input):

    topic = input
    blog_post = create_a_blog_post(topic)
    prompt = get_prompt(blog_post)
    images = get_image_from_prompt(prompt)

    return {"blog_post": blog_post, "images": images}


@coprocessor(type="inference", model="bloom", id="bloom-call")
def bloom(prompt, model):

    result = model(prompt)

    return result


@smartpipe(path="/bloom", method="POST", id="my-bloom-smart-pipe")
def bloom_smartpipe(input_data=None):

    return bloom(input_data)


@coprocessor(type="inference", model="stable-diffusion", id="stable-diffusion-model")
def stable_diffusion(prompt, model):
    result = model(prompt)

    return result


@smartpipe(path="/stable_diffusion", method="POST", id="stable-diffusion-smartpipe")
def stable_diffusion_smartpipe(input_data=None):

    return stable_diffusion(input_data)


app = start()
