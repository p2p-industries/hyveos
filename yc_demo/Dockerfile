ARG BRANCH=main

FROM ghcr.io/p2p-industries/hyveos-base-python:${BRANCH}

RUN apt-get update
RUN apt-get install --yes build-essential

WORKDIR /app/hello_world

COPY poetry.lock poetry.lock
COPY pyproject.toml pyproject.toml

RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --no-root

COPY main.py main.py

ENV PYTHONUNBUFFERED=1

CMD ["poetry", "run", "python", "main.py"]
