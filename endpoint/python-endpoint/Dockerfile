FROM ghcr.io/p2p-industries/mvp-base-python:pr-73

WORKDIR /app

COPY . /app/python-endpoint/

WORKDIR /app/python-endpoint/

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

ENV PYTHON_ENDPOINT_PORT=8000

EXPOSE 8000/tcp

CMD ["poetry", "run", "python", "server.py"]
