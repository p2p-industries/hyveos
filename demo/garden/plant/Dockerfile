FROM ghcr.io/p2p-industries/mvp-base-python:main

RUN apt update
RUN apt install --yes build-essential

COPY poetry.lock /app/demo/garden/plant/poetry.lock
COPY pyproject.toml /app/demo/garden/plant/pyproject.toml

WORKDIR /app/demo/garden/plant

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY plant /app/demo/garden/plant/plant

ENV PYTHONUNBUFFERED=1

LABEL industries.p2p.script.name=plant1

ENV PLANT_1=1
ENV PLANT_2=1
ENV PLANT_3=1

CMD ["poetry", "run", "python", "-m", "plant.plant"]
