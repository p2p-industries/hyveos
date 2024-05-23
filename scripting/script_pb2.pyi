from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ID(_message.Message):
    __slots__ = ("value",)
    VALUE_FIELD_NUMBER: _ClassVar[int]
    value: str
    def __init__(self, value: _Optional[str] = ...) -> None: ...

class Message(_message.Message):
    __slots__ = ("data", "topic")
    DATA_FIELD_NUMBER: _ClassVar[int]
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    topic: str
    def __init__(self, data: _Optional[bytes] = ..., topic: _Optional[str] = ...) -> None: ...

class SendRequest(_message.Message):
    __slots__ = ("peer_id", "msg")
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    msg: Message
    def __init__(self, peer_id: _Optional[str] = ..., msg: _Optional[_Union[Message, _Mapping]] = ...) -> None: ...

class RecvRequest(_message.Message):
    __slots__ = ("peer_id", "msg", "seq")
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    SEQ_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    msg: Message
    seq: int
    def __init__(self, peer_id: _Optional[str] = ..., msg: _Optional[_Union[Message, _Mapping]] = ..., seq: _Optional[int] = ...) -> None: ...

class Topic(_message.Message):
    __slots__ = ("topic",)
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    topic: str
    def __init__(self, topic: _Optional[str] = ...) -> None: ...

class Response(_message.Message):
    __slots__ = ("data",)
    DATA_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    def __init__(self, data: _Optional[bytes] = ...) -> None: ...

class SendResponse(_message.Message):
    __slots__ = ("seq", "response")
    SEQ_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    seq: int
    response: Response
    def __init__(self, seq: _Optional[int] = ..., response: _Optional[_Union[Response, _Mapping]] = ...) -> None: ...

class Peer(_message.Message):
    __slots__ = ("peer_id",)
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    def __init__(self, peer_id: _Optional[str] = ...) -> None: ...

class Peers(_message.Message):
    __slots__ = ("peers",)
    PEERS_FIELD_NUMBER: _ClassVar[int]
    peers: _containers.RepeatedCompositeFieldContainer[Peer]
    def __init__(self, peers: _Optional[_Iterable[_Union[Peer, _Mapping]]] = ...) -> None: ...

class NeighbourEvent(_message.Message):
    __slots__ = ("discovered", "lost")
    DISCOVERED_FIELD_NUMBER: _ClassVar[int]
    LOST_FIELD_NUMBER: _ClassVar[int]
    discovered: Peer
    lost: Peer
    def __init__(self, discovered: _Optional[_Union[Peer, _Mapping]] = ..., lost: _Optional[_Union[Peer, _Mapping]] = ...) -> None: ...

class MeshTopologyInit(_message.Message):
    __slots__ = ("peer", "neighbours")
    PEER_FIELD_NUMBER: _ClassVar[int]
    NEIGHBOURS_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    neighbours: Peers
    def __init__(self, peer: _Optional[_Union[Peer, _Mapping]] = ..., neighbours: _Optional[_Union[Peers, _Mapping]] = ...) -> None: ...

class MeshTopologyUpdate(_message.Message):
    __slots__ = ("peer", "event")
    PEER_FIELD_NUMBER: _ClassVar[int]
    EVENT_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    event: NeighbourEvent
    def __init__(self, peer: _Optional[_Union[Peer, _Mapping]] = ..., event: _Optional[_Union[NeighbourEvent, _Mapping]] = ...) -> None: ...

class MeshTopologyEvent(_message.Message):
    __slots__ = ("init", "update")
    INIT_FIELD_NUMBER: _ClassVar[int]
    UPDATE_FIELD_NUMBER: _ClassVar[int]
    init: MeshTopologyInit
    update: MeshTopologyUpdate
    def __init__(self, init: _Optional[_Union[MeshTopologyInit, _Mapping]] = ..., update: _Optional[_Union[MeshTopologyUpdate, _Mapping]] = ...) -> None: ...

class GossipSubMessage(_message.Message):
    __slots__ = ("peer_id", "msg")
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    msg: Message
    def __init__(self, peer_id: _Optional[str] = ..., msg: _Optional[_Union[Message, _Mapping]] = ...) -> None: ...

class DHTKey(_message.Message):
    __slots__ = ("key",)
    KEY_FIELD_NUMBER: _ClassVar[int]
    key: str
    def __init__(self, key: _Optional[str] = ...) -> None: ...

class DHTRecord(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: DHTKey
    value: bytes
    def __init__(self, key: _Optional[_Union[DHTKey, _Mapping]] = ..., value: _Optional[bytes] = ...) -> None: ...

class FilePath(_message.Message):
    __slots__ = ("path",)
    PATH_FIELD_NUMBER: _ClassVar[int]
    path: str
    def __init__(self, path: _Optional[str] = ...) -> None: ...

class File(_message.Message):
    __slots__ = ("path", "cid")
    PATH_FIELD_NUMBER: _ClassVar[int]
    CID_FIELD_NUMBER: _ClassVar[int]
    path: FilePath
    cid: ID
    def __init__(self, path: _Optional[_Union[FilePath, _Mapping]] = ..., cid: _Optional[_Union[ID, _Mapping]] = ...) -> None: ...
