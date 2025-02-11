FROM python:3.11-slim AS poetry-builder

ENV POETRY_HOME=/opt/poetry

RUN --mount=target=/var/lib/apt/lists,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean \
    && apt-get update \
    && apt-get -y --no-install-recommends install \
    curl

RUN curl -sSL https://install.python-poetry.org | python3 -

FROM python:3.11-slim AS builder

ENV POETRY_HOME=/opt/poetry
COPY --from=poetry-builder ${POETRY_HOME} ${POETRY_HOME}

ENV PATH="$POETRY_HOME/bin:$PATH"

RUN echo $PATH

WORKDIR /app/python/sdks

COPY poetry.lock pyproject.toml ./

RUN poetry install --no-root

COPY hyveos_sdk ./hyveos_sdk

RUN poetry install --only main
RUN poetry build

FROM python:3.11-slim

ENV POETRY_HOME=/opt/poetry
ENV PATH="$POETRY_HOME/bin:$PATH"
COPY --from=poetry-builder ${POETRY_HOME} ${POETRY_HOME}
COPY --from=builder /app/python/sdks/dist/*.whl /app/python/sdks/dist/

LABEL org.opencontainers.image.title="hyveOS Python SDK"
LABEL org.opencontainers.image.description="hyveOS Python SDK"
LABEL org.opencontainers.image.documentation="https://docs.p2p.industries"
LABEL org.opencontainers.image.licenses="MIT"
