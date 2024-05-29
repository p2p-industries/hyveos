FROM python:alpine

RUN apk add build-base musl-dev libffi-dev protobuf

RUN python3 -m pip install poetry

WORKDIR /app

COPY sdks/python/ /app/sdks/python/
COPY demo/garden/pump/ /app/demo/garden/pump/

WORKDIR /app/demo/garden/pump
RUN poetry install --only main

CMD ["poetry", "run", "python", "main.py"]
