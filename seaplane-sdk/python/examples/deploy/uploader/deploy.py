import os

# needs to export VAULT_ADDR
_SMART_PIPE_FILE_NAME = "SMART_PIPE_FILE_NAME"
_SMART_PIPE_FOLDER = "SMART_PIPE_FOLDER"
_SMART_PIPE_USER = "SMART_PIPE_USER"
_VAULT_TOKEN = "VAULT_TOKEN"

REGIONS = ["fra"]

dockerfile = f"""
FROM python:3.10

RUN \
	apt update && apt install -y wget unzip \
	&& wget https://releases.hashicorp.com/vault/1.6.2/vault_1.6.2_linux_amd64.zip \
	&& unzip vault_1.6.2_linux_amd64.zip	

ENV SEAPLANE_PRODUCTION True

WORKDIR /app
COPY . . 

RUN pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn


EXPOSE 1337

#RUN ["python", "demo.py"]
CMD ["gunicorn", "--config", "gunicorn_config.py", "{os.getenv(_SMART_PIPE_FILE_NAME)}:app"]
"""


def login():
    os.system(f"vault login {os.getenv(_VAULT_TOKEN)}")
    os.system(
        "vault kv get -mount secret -field github_artifact_writer github | \
      docker login \
        -u _json_key \
        --password-stdin \
        https://us-central1-docker.pkg.dev"
    )


def build_image(folder, image_name):
    os.system(f"docker build -t docker-image {folder}")
    os.system(
        f"docker tag docker-image  us-central1-docker.pkg.dev/artifacts-356722/demo/{image_name}:latest"
    )
    os.system(f"docker push us-central1-docker.pkg.dev/artifacts-356722/demo/{image_name}:latest")


def deploy(customer):
    for region in REGIONS:
        os.system(f"nomad job run --region={region} {customer}.nomad.hcl")


login()
build_image(os.getenv(_SMART_PIPE_FOLDER), os.getenv(_SMART_PIPE_USER))
deploy()
