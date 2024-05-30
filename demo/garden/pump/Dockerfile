FROM base-python:slim

WORKDIR /app

COPY . /app/demo/garden/pump/

WORKDIR /app/demo/garden/pump

RUN poetry remove p2pindustries
RUN poetry add /app/python/sdks/dist/*.whl
RUN poetry install --only main

CMD ["poetry", "run", "python", "main.py"]
