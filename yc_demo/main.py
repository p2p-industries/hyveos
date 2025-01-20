import asyncio
import json
from datetime import datetime
from hyveos_sdk import Connection


async def main():
    print("STATUS SCRIPT is RUNNING ---")
    async with Connection() as connection:
        discovery = connection.get_discovery_service()
        gossip_sub = connection.get_gossip_sub_service()

        own_id = await discovery.get_own_id()

        while True:
            now = datetime.now().isoformat()

            message = {
                "id": own_id,
                "timestamp": now,
                "status": "OK",
                "sequence": 2048,
            }

            json_pretty = json.dumps(message, indent=4, sort_keys=True)

            await gossip_sub.publish(json_pretty, "status")

            await asyncio.sleep(5)


if __name__ == "__main__":
    asyncio.run(main())
