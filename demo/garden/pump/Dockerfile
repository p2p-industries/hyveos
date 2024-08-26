ARG BRANCH=main

FROM ghcr.io/p2p-industries/mvp-base-python:${BRANCH}

COPY poetry.lock /app/demo/garden/pump/poetry.lock
COPY pyproject.toml /app/demo/garden/pump/pyproject.toml

WORKDIR /app/demo/garden/pump


RUN poetry remove p2pindustries
RUN --mount=type=cache,target=/opt/.cache/pypoetry poetry add /app/python/sdks/dist/*.whl
RUN --mount=type=cache,target=/opt/.cache/pypoetry poetry install --only main

COPY main.py /app/demo/garden/pump/main.py

ENV PYTHONUNBUFFERED=1

ENV FLOW_METER_PIN=22
ENV PWM_PIN=18
ENV FWD_PIN=23
ENV REV_PIN=24

CMD ["poetry", "run", "python", "main.py"]
