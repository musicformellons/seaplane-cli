FROM python:3.10

ENV SMARTPIPES_PRODUCTION True

WORKDIR /app
COPY . . 

RUN pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn


EXPOSE 1337

#RUN ["python", "demo.py"]
CMD ["gunicorn", "--config", "gunicorn_config.py", "demo:app"]
