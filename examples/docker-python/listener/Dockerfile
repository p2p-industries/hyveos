FROM ghcr.io/p2p-industries/hyveos-base-python:main

COPY ./pyproject.toml /app/client/pyproject.toml

WORKDIR /app/client

RUN poetry remove hyveos-sdk
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY ./script.py /app/client/script.py

ENV PYTHONUNBUFFERED=1

CMD ["poetry", "run", "python", "script.py"]

LABEL org.opencontainers.image.title="hyveOS Python example image (listener)"
LABEL org.opencontainers.image.url="https://docs.p2p.industries"
LABEL org.opencontainers.image.documentation="https://docs.p2p.industries"
LABEL org.opencontainers.image.licenses="GPL-3.0-only"
