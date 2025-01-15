ARG BRANCH=main

FROM ghcr.io/p2p-industries/hyveos-base-python:${BRANCH}

RUN apt-get update
RUN apt-get install --yes build-essential

WORKDIR /app/demo/garden/plant

COPY poetry.lock poetry.lock
COPY pyproject.toml pyproject.toml

RUN poetry remove hyveos-sdk
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --without dev --no-root

COPY plant plant

ENV PYTHONUNBUFFERED=1

LABEL industries.p2p.app.name=plant

CMD ["poetry", "run", "python", "-m", "plant.main"]
