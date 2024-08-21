FROM ghcr.io/p2p-industries/mvp-base-python:main

RUN apt update
RUN apt install --yes build-essential

COPY poetry.lock /app/demo/garden/plant/poetry.lock
COPY pyproject.toml /app/demo/garden/plant/pyproject.toml

WORKDIR /app/demo/garden/plant

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY plant.py /app/demo/garden/plant/plant.py

ENV PYTHONUNBUFFERED=1

ENV VALVE_PIN=6

CMD ["poetry", "run", "python", "plant.py"]
