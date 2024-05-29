FROM python:3.11-slim AS poetry-builder

ARG POETRY_HOME
RUN pip install poetry

FROM python:3.11-slim AS builder

COPY --from=poetry-builder ${POETRY_HOME} ${POETRY_HOME}

WORKDIR /app/python/sdks
COPY . .
RUN poetry install --no-dev
RUN poetry build

FROM python:3.11-slim

COPY --from=poetry-builder ${POETRY_HOME} ${POETRY_HOME}
COPY --from=builder /app/python/sdks/dist/*.whl /app/python/sdks/dist/
