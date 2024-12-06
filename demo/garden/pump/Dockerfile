ARG BRANCH=main

FROM ghcr.io/p2p-industries/hyveos-base-python:${BRANCH}

WORKDIR /app/demo/garden/pump

COPY poetry.lock poetry.lock
COPY pyproject.toml pyproject.toml

RUN poetry remove hyveos-sdk
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --without dev --no-root

COPY main.py main.py

ENV PYTHONUNBUFFERED=1

ENV FLOW_METER_PIN=22
ENV PWM_PIN=18
ENV FWD_PIN=23
ENV REV_PIN=24

CMD ["poetry", "run", "python", "main.py"]
