import os
import asyncio
import json

from p2pindustries import P2PConnection
from prometheus_client import Gauge, start_http_server


def create_bridge(gauges, reqres, topic):

    async def bridge():
        async with reqres.receive(topic) as requests:
            async for request in requests:
                try:
                    msg = json.loads(request.msg.data)
                    gauges[topic].set(msg[topic])
                    await reqres.respond(request.seq, json.dumps({'status': 'success'}))
                except Exception as e:
                    await reqres.respond(request.seq, json.dumps({'status': 'failed'}), str(e))

    return bridge


async def main(port, metric_names):
    async with P2PConnection() as connection:
        dht = connection.get_dht_service()
        reqres = connection.get_request_response_service()

        await dht.provide('identification', 'prometheus')

        gauges = {name: Gauge(name, f"GossipSub Topic {name}") for name in metric_names}

        start_http_server(port)

        producers = [create_bridge(gauges, reqres, name)() for name in metric_names]
        await asyncio.gather(*producers)


if __name__ == '__main__':
    port = os.environ['PROMETHEUS_EXPORTER_PORT']
    topics = os.environ['PROMETHEUS_EXPORTER_TOPICS'].split(',')

    asyncio.run(main(int(port), topics))