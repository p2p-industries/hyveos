ARG BRANCH=main

FROM ghcr.io/p2p-industries/mvp-base-python:${BRANCH}

RUN apt-get update
RUN apt-get install --yes build-essential

WORKDIR /app/demo/garden/plant

COPY poetry.lock poetry.lock
COPY pyproject.toml pyproject.toml

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY plant plant

ENV PYTHONUNBUFFERED=1

LABEL industries.p2p.script.name=plant1

ENV PLANT_1=1
ENV MOISTURE_THRESHOLD_1=15
ENV PLANT_2=2
ENV MOISTURE_THRESHOLD_2=15
ENV PLANT_3=3
ENV MOISTURE_THRESHOLD_3=15

CMD ["poetry", "run", "python", "-m", "plant.plant"]
