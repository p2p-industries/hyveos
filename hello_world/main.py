import asyncio
from hyveos_sdk import Connection


async def wait_for_peers(discovery):
    await asyncio.sleep(5)
    async for event in discovery.discovery_events():
        if event is not None:
            print(f"Discovered a peer: {event}")
            break


async def node(topic):
    print("--- APPLICATION SCRIPT is RUNNING ---")
    async with Connection() as connection:
        discovery = connection.get_discovery_service()
        gossip_sub = connection.get_gossip_sub_service()

        own_id = await discovery.get_own_id()
        print(f"Node {own_id} has started.")

        while True:
            await wait_for_peers(discovery)

            stream = await gossip_sub.subscribe(topic)

            await asyncio.sleep(12)

            message = f"Hello from {own_id}"
            await gossip_sub.publish(message, topic)

            async for msg in stream:
                if msg != message:
                    print(f"Node {own_id} received: {msg}")
                    break


async def main():
    topic = "greetings"
    await node(topic)


if __name__ == "__main__":
    asyncio.run(main())
