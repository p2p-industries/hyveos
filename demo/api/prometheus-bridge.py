import asyncio
import random
import argparse

from prometheus_client import Gauge, start_http_server


def create_bridge(name, gauge):
    async def bridge(name):
        async for data in gossipSub.recv(topic_name):
            gauge.set(data)

    return bridge


async def main(port, metric_names):
    gauges = {name: Gauge(name, f"GossipSub Topic {name}") for name in metric_names}

    start_http_server(port)

    producers = [create_bridge(name, gauges[name])() for name in metric_names]
    await asyncio.gather(*producers)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Prometheus Bridge to export GossipSub Topics to a Prometheus Database')
    parser.add_argument('--topics', metavar='M', type=str, nargs='+',
                        help='a list of gossipSub topics to export', required=True)
    parser.add_argument('--port', type=int, default=8000, help='Port to expose')

    args = parser.parse_args()

    asyncio.run(main(args.port, args.topics))
