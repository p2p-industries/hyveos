FROM ghcr.io/p2p-industries/mvp-base-python:pr-73

WORKDIR /app

COPY . /app/prometheus-exporter/

WORKDIR /app/prometheus-exporter/

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

ENV PROMETHEUS_EXPORTER_PORT=8090
ENV PROMETHEUS_EXPORTER_TOPICS=temperature,humidity,lux,soil_moisture

EXPOSE 8090/tcp

CMD ["poetry", "run", "python", "exporter.py"]