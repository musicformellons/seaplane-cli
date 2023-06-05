FROM docker:stable-dind

# Install dependencies (if needed)
RUN apk update && apk add --no-cache \
  curl \
  wget \ 
  unzip \
  gcc \
  libc-dev \
  libffi-dev \
  openssl-dev

RUN \	
	wget https://releases.hashicorp.com/vault/1.6.2/vault_1.6.2_linux_amd64.zip \
	&& unzip vault_1.6.2_linux_amd64.zip	

# Install Python
RUN apk add --no-cache python3 python3-dev

# Set Docker environment variables (if needed)
#ENV DOCKER_TLS_CERTDIR=""
#ENV DOCKER_HOST="unix:///var/run/docker.sock"

ENV SEAPLANE_PRODUCTION True

WORKDIR /app
COPY . . 

RUN pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn


EXPOSE 1337

#RUN ["python", "demo.py"]
CMD ["gunicorn", "--config", "gunicorn_config.py", "demo:app"]
