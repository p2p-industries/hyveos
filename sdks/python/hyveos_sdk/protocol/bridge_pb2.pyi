from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Data(_message.Message):
    __slots__ = ("data",)
    DATA_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    def __init__(self, data: _Optional[bytes] = ...) -> None: ...

class OptionalData(_message.Message):
    __slots__ = ("data",)
    DATA_FIELD_NUMBER: _ClassVar[int]
    data: Data
    def __init__(self, data: _Optional[_Union[Data, _Mapping]] = ...) -> None: ...

class ID(_message.Message):
    __slots__ = ("ulid",)
    ULID_FIELD_NUMBER: _ClassVar[int]
    ulid: str
    def __init__(self, ulid: _Optional[str] = ...) -> None: ...

class Peer(_message.Message):
    __slots__ = ("peer_id",)
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    def __init__(self, peer_id: _Optional[str] = ...) -> None: ...

class Topic(_message.Message):
    __slots__ = ("topic",)
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    topic: str
    def __init__(self, topic: _Optional[str] = ...) -> None: ...

class OptionalTopic(_message.Message):
    __slots__ = ("topic",)
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    topic: Topic
    def __init__(self, topic: _Optional[_Union[Topic, _Mapping]] = ...) -> None: ...

class TopicQuery(_message.Message):
    __slots__ = ("topic", "regex")
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    REGEX_FIELD_NUMBER: _ClassVar[int]
    topic: Topic
    regex: str
    def __init__(self, topic: _Optional[_Union[Topic, _Mapping]] = ..., regex: _Optional[str] = ...) -> None: ...

class OptionalTopicQuery(_message.Message):
    __slots__ = ("query",)
    QUERY_FIELD_NUMBER: _ClassVar[int]
    query: TopicQuery
    def __init__(self, query: _Optional[_Union[TopicQuery, _Mapping]] = ...) -> None: ...

class Message(_message.Message):
    __slots__ = ("data", "topic")
    DATA_FIELD_NUMBER: _ClassVar[int]
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    data: Data
    topic: OptionalTopic
    def __init__(self, data: _Optional[_Union[Data, _Mapping]] = ..., topic: _Optional[_Union[OptionalTopic, _Mapping]] = ...) -> None: ...

class SendRequest(_message.Message):
    __slots__ = ("peer", "msg")
    PEER_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    msg: Message
    def __init__(self, peer: _Optional[_Union[Peer, _Mapping]] = ..., msg: _Optional[_Union[Message, _Mapping]] = ...) -> None: ...

class RecvRequest(_message.Message):
    __slots__ = ("peer", "msg", "seq")
    PEER_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    SEQ_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    msg: Message
    seq: int
    def __init__(self, peer: _Optional[_Union[Peer, _Mapping]] = ..., msg: _Optional[_Union[Message, _Mapping]] = ..., seq: _Optional[int] = ...) -> None: ...

class Response(_message.Message):
    __slots__ = ("data", "error")
    DATA_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    data: Data
    error: str
    def __init__(self, data: _Optional[_Union[Data, _Mapping]] = ..., error: _Optional[str] = ...) -> None: ...

class SendResponse(_message.Message):
    __slots__ = ("seq", "response")
    SEQ_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    seq: int
    response: Response
    def __init__(self, seq: _Optional[int] = ..., response: _Optional[_Union[Response, _Mapping]] = ...) -> None: ...

class Peers(_message.Message):
    __slots__ = ("peers",)
    PEERS_FIELD_NUMBER: _ClassVar[int]
    peers: _containers.RepeatedCompositeFieldContainer[Peer]
    def __init__(self, peers: _Optional[_Iterable[_Union[Peer, _Mapping]]] = ...) -> None: ...

class NeighbourEvent(_message.Message):
    __slots__ = ("init", "discovered", "lost")
    INIT_FIELD_NUMBER: _ClassVar[int]
    DISCOVERED_FIELD_NUMBER: _ClassVar[int]
    LOST_FIELD_NUMBER: _ClassVar[int]
    init: Peers
    discovered: Peer
    lost: Peer
    def __init__(self, init: _Optional[_Union[Peers, _Mapping]] = ..., discovered: _Optional[_Union[Peer, _Mapping]] = ..., lost: _Optional[_Union[Peer, _Mapping]] = ...) -> None: ...

class PubSubMessageID(_message.Message):
    __slots__ = ("id",)
    ID_FIELD_NUMBER: _ClassVar[int]
    id: bytes
    def __init__(self, id: _Optional[bytes] = ...) -> None: ...

class PubSubMessage(_message.Message):
    __slots__ = ("data", "topic")
    DATA_FIELD_NUMBER: _ClassVar[int]
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    data: Data
    topic: Topic
    def __init__(self, data: _Optional[_Union[Data, _Mapping]] = ..., topic: _Optional[_Union[Topic, _Mapping]] = ...) -> None: ...

class PubSubRecvMessage(_message.Message):
    __slots__ = ("propagation_source", "source", "msg", "msg_id")
    PROPAGATION_SOURCE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    MSG_ID_FIELD_NUMBER: _ClassVar[int]
    propagation_source: Peer
    source: Peer
    msg: PubSubMessage
    msg_id: PubSubMessageID
    def __init__(self, propagation_source: _Optional[_Union[Peer, _Mapping]] = ..., source: _Optional[_Union[Peer, _Mapping]] = ..., msg: _Optional[_Union[PubSubMessage, _Mapping]] = ..., msg_id: _Optional[_Union[PubSubMessageID, _Mapping]] = ...) -> None: ...

class DHTKey(_message.Message):
    __slots__ = ("topic", "key")
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    KEY_FIELD_NUMBER: _ClassVar[int]
    topic: Topic
    key: bytes
    def __init__(self, topic: _Optional[_Union[Topic, _Mapping]] = ..., key: _Optional[bytes] = ...) -> None: ...

class DHTRecord(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: DHTKey
    value: Data
    def __init__(self, key: _Optional[_Union[DHTKey, _Mapping]] = ..., value: _Optional[_Union[Data, _Mapping]] = ...) -> None: ...

class LocalKVRecord(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Data
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Data, _Mapping]] = ...) -> None: ...

class LocalKVKey(_message.Message):
    __slots__ = ("key",)
    KEY_FIELD_NUMBER: _ClassVar[int]
    key: str
    def __init__(self, key: _Optional[str] = ...) -> None: ...

class FilePath(_message.Message):
    __slots__ = ("path",)
    PATH_FIELD_NUMBER: _ClassVar[int]
    path: str
    def __init__(self, path: _Optional[str] = ...) -> None: ...

class CID(_message.Message):
    __slots__ = ("hash", "id")
    HASH_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    hash: bytes
    id: ID
    def __init__(self, hash: _Optional[bytes] = ..., id: _Optional[_Union[ID, _Mapping]] = ...) -> None: ...

class DownloadEvent(_message.Message):
    __slots__ = ("progress", "ready")
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    progress: int
    ready: FilePath
    def __init__(self, progress: _Optional[int] = ..., ready: _Optional[_Union[FilePath, _Mapping]] = ...) -> None: ...

class MeshTopologyEvent(_message.Message):
    __slots__ = ("peer", "event")
    PEER_FIELD_NUMBER: _ClassVar[int]
    EVENT_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    event: NeighbourEvent
    def __init__(self, peer: _Optional[_Union[Peer, _Mapping]] = ..., event: _Optional[_Union[NeighbourEvent, _Mapping]] = ...) -> None: ...

class RequestDebugEvent(_message.Message):
    __slots__ = ("id", "receiver", "msg")
    ID_FIELD_NUMBER: _ClassVar[int]
    RECEIVER_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    id: ID
    receiver: Peer
    msg: Message
    def __init__(self, id: _Optional[_Union[ID, _Mapping]] = ..., receiver: _Optional[_Union[Peer, _Mapping]] = ..., msg: _Optional[_Union[Message, _Mapping]] = ...) -> None: ...

class ResponseDebugEvent(_message.Message):
    __slots__ = ("req_id", "response")
    REQ_ID_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    req_id: ID
    response: Response
    def __init__(self, req_id: _Optional[_Union[ID, _Mapping]] = ..., response: _Optional[_Union[Response, _Mapping]] = ...) -> None: ...

class MessageDebugEvent(_message.Message):
    __slots__ = ("sender", "req", "res", "pub_sub")
    SENDER_FIELD_NUMBER: _ClassVar[int]
    REQ_FIELD_NUMBER: _ClassVar[int]
    RES_FIELD_NUMBER: _ClassVar[int]
    PUB_SUB_FIELD_NUMBER: _ClassVar[int]
    sender: Peer
    req: RequestDebugEvent
    res: ResponseDebugEvent
    pub_sub: PubSubMessage
    def __init__(self, sender: _Optional[_Union[Peer, _Mapping]] = ..., req: _Optional[_Union[RequestDebugEvent, _Mapping]] = ..., res: _Optional[_Union[ResponseDebugEvent, _Mapping]] = ..., pub_sub: _Optional[_Union[PubSubMessage, _Mapping]] = ...) -> None: ...

class DockerImage(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class DockerApp(_message.Message):
    __slots__ = ("image", "ports")
    IMAGE_FIELD_NUMBER: _ClassVar[int]
    PORTS_FIELD_NUMBER: _ClassVar[int]
    image: DockerImage
    ports: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, image: _Optional[_Union[DockerImage, _Mapping]] = ..., ports: _Optional[_Iterable[int]] = ...) -> None: ...

class DeployAppRequest(_message.Message):
    __slots__ = ("app", "local", "peer", "persistent")
    APP_FIELD_NUMBER: _ClassVar[int]
    LOCAL_FIELD_NUMBER: _ClassVar[int]
    PEER_FIELD_NUMBER: _ClassVar[int]
    PERSISTENT_FIELD_NUMBER: _ClassVar[int]
    app: DockerApp
    local: bool
    peer: Peer
    persistent: bool
    def __init__(self, app: _Optional[_Union[DockerApp, _Mapping]] = ..., local: bool = ..., peer: _Optional[_Union[Peer, _Mapping]] = ..., persistent: bool = ...) -> None: ...

class ListRunningAppsRequest(_message.Message):
    __slots__ = ("peer",)
    PEER_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    def __init__(self, peer: _Optional[_Union[Peer, _Mapping]] = ...) -> None: ...

class RunningApp(_message.Message):
    __slots__ = ("id", "image", "name")
    ID_FIELD_NUMBER: _ClassVar[int]
    IMAGE_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    id: ID
    image: DockerImage
    name: str
    def __init__(self, id: _Optional[_Union[ID, _Mapping]] = ..., image: _Optional[_Union[DockerImage, _Mapping]] = ..., name: _Optional[str] = ...) -> None: ...

class RunningApps(_message.Message):
    __slots__ = ("apps",)
    APPS_FIELD_NUMBER: _ClassVar[int]
    apps: _containers.RepeatedCompositeFieldContainer[RunningApp]
    def __init__(self, apps: _Optional[_Iterable[_Union[RunningApp, _Mapping]]] = ...) -> None: ...

class StopAppRequest(_message.Message):
    __slots__ = ("id", "peer")
    ID_FIELD_NUMBER: _ClassVar[int]
    PEER_FIELD_NUMBER: _ClassVar[int]
    id: ID
    peer: Peer
    def __init__(self, id: _Optional[_Union[ID, _Mapping]] = ..., peer: _Optional[_Union[Peer, _Mapping]] = ...) -> None: ...
