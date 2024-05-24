import grpc
from typing import TypeVar, Generic

T = TypeVar('T')


class ManagedStream(Generic[T]):
    def __init__(self, stream):
        self.stream = stream

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.stream.cancel()

        try:
            await self.stream
        except grpc.aio.AioRpcError as e:
            if e.code() != grpc.StatusCode.CANCELLED:
                print(f'Stream Cancellation error: {e}')

    def __aiter__(self):
        return self

    async def __anext__(self) -> T:
        try:
            return await self.stream.__anext__()
        except StopAsyncIteration:
            raise StopAsyncIteration
