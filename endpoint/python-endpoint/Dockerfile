FROM ghcr.io/p2p-industries/mvp-base-python:main

COPY poetry.lock /app/python-endpoint/poetry.lock
COPY pyproject.toml /app/python-endpoint/pyproject.toml

WORKDIR /app/python-endpoint/

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY server.py /app/python-endpoint/server.py

ENV PYTHONUNBUFFERED=1

ENV PYTHON_ENDPOINT_PORT=8000

EXPOSE 8000/tcp

CMD ["poetry", "run", "python", "server.py"]
