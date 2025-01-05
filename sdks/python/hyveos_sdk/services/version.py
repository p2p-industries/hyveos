from grpc.aio import Channel
from ..protocol.script_pb2_grpc import BridgeInfoStub
from ..protocol.script_pb2 import Version as InnerVersion, Empty

class Version:
    major: int
    minor: int
    patch: int
    pre_release: str | None

    def __init__(self, version: InnerVersion):
        self.major = version.major
        self.minor = version.minor
        self.patch = version.patch
        self.pre_release = version.pre

    def __str__(self):
        return f"{self.major}.{self.minor}.{self.patch}" + (f"-{self.pre_release}" if self.pre_release else "")

    def __repr__(self):
        return f"Version(major={self.major}, minor={self.minor}, patch={self.patch}, pre_release={self.pre_release})"

    def __eq__(self, other):
        return self.major == other.major and self.minor == other.minor and self.patch == other.patch and self.pre_release == other.pre_release

class VersionService:
    """
    Retrieve the version of the runtime
    """

    def __init__(self, conn: Channel):
        self.stub = BridgeInfoStub(conn)

    async def __get_version(self) -> InnerVersion:
        return await self.stub.GetVersion(Empty())

    async def get_version(self) -> Version:
        """
        Returns the version of the runtime
        """
        inner_version = await self.__get_version()
        return Version(inner_version)

