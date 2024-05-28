FROM python:3.10

WORKDIR /p2p/script

COPY ./sdks/python/p2pindustries ./packages/p2pindustries
COPY ./endpoint/python-endpoint/requirements.txt .

RUN pip install --no-cache-dir -r requirements.txt

COPY ./endpoint/python-endpoint/server.py .

ENV PYTHONUNBUFFERED=1
ENV PYTHONPATH=/p2p/script/packages

EXPOSE 8000

CMD [ "python", "./server.py" ]
