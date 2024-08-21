FROM ghcr.io/p2p-industries/mvp-base-python:main

COPY poetry.lock /app/prometheus-exporter/poetry.lock
COPY pyproject.toml /app/prometheus-exporter/pyproject.toml

WORKDIR /app/prometheus-exporter/

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY exporter.py /app/prometheus-exporter/exporter.py

ENV PYTHONUNBUFFERED=1

ENV PROMETHEUS_EXPORTER_PORT=8090
ENV PROMETHEUS_EXPORTER_TOPICS=temperature,humidity,lux,soil_moisture,flow_meter

EXPOSE 8090/tcp

CMD ["poetry", "run", "python", "exporter.py"]
