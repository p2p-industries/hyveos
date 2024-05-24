class ManagedStream:
    def __init__(self, stream):
        self.stream = stream

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.stream.cancel()

    def __aiter__(self):
        return self

    async def __anext__(self):
        try:
            return await self.stream.__anext__()
        except StopAsyncIteration:
            raise StopAsyncIteration
