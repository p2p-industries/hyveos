FROM ghcr.io/p2p-industries/mvp-base-python:main

WORKDIR /app

COPY . /app/demo/garden/pump/

WORKDIR /app/demo/garden/pump

RUN apt update
RUN apt install --yes build-essential

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

CMD ["poetry", "run", "python", "main.py"]
