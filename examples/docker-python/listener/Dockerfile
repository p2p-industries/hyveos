FROM ghcr.io/p2p-industries/hyveos-base-python:main

COPY ./pyproject.toml /app/client/pyproject.toml

WORKDIR /app/client

RUN poetry remove hyveos-sdk
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

COPY ./script.py /app/client/script.py

ENV PYTHONUNBUFFERED=1

CMD ["poetry", "run", "python", "script.py"]
