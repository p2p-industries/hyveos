FROM python:3.10-alpine

WORKDIR /p2p/script

COPY ./sdks/python/p2pindustries ./packages/p2pindustries
COPY ./examples/docker-python/listener/requirements.txt .

RUN pip install --no-cache-dir -r requirements.txt

COPY ./examples/docker-python/listener/script.py .

ENV PYTHONUNBUFFERED=1
ENV PYTHONPATH=/p2p/script/packages

CMD [ "python", "./script.py" ]
