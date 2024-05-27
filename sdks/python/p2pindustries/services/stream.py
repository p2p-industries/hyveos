from typing import AsyncIterator, TypeVar, Generic

T = TypeVar('T')


class ManagedStream(Generic[T]):
    def __init__(self, stream):
        self.stream = stream

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.stream.cancel()

    def __aiter__(self) -> AsyncIterator[T]:
        return self.stream.__aiter__()
