FROM ghcr.io/p2p-industries/mvp-base-python:main

WORKDIR /app

COPY . /app/demo/garden/pump/

WORKDIR /app/demo/garden/pump

# RUN apt install --yes build-essentials

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

ENV FLOW_METER_PIN=22
ENV PUMP_PIN=13

CMD ["poetry", "run", "python", "main.py"]
