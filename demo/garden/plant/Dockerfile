FROM ghcr.io/p2p-industries/mvp-base-python:pr-73

WORKDIR /app

COPY . /app/demo/garden/plant/

WORKDIR /app/demo/garden/plant

RUN apt update
RUN apt install --yes build-essential

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

CMD ["poetry", "run", "python", "plant.py"]