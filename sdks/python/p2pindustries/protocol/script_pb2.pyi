from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import (
    ClassVar as _ClassVar,
    Iterable as _Iterable,
    Mapping as _Mapping,
    Optional as _Optional,
    Union as _Union,
)

DESCRIPTOR: _descriptor.FileDescriptor

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ID(_message.Message):
    __slots__ = ('ulid',)
    ULID_FIELD_NUMBER: _ClassVar[int]
    ulid: str
    def __init__(self, ulid: _Optional[str] = ...) -> None: ...

class Topic(_message.Message):
    __slots__ = ('topic',)
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    topic: str
    def __init__(self, topic: _Optional[str] = ...) -> None: ...

class OptionalTopic(_message.Message):
    __slots__ = ('topic',)
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    topic: Topic
    def __init__(self, topic: _Optional[_Union[Topic, _Mapping]] = ...) -> None: ...

class TopicQuery(_message.Message):
    __slots__ = ('topic', 'regex')
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    REGEX_FIELD_NUMBER: _ClassVar[int]
    topic: Topic
    regex: str
    def __init__(
        self,
        topic: _Optional[_Union[Topic, _Mapping]] = ...,
        regex: _Optional[str] = ...,
    ) -> None: ...

class OptionalTopicQuery(_message.Message):
    __slots__ = ('query',)
    QUERY_FIELD_NUMBER: _ClassVar[int]
    query: TopicQuery
    def __init__(
        self, query: _Optional[_Union[TopicQuery, _Mapping]] = ...
    ) -> None: ...

class Message(_message.Message):
    __slots__ = ('data', 'topic')
    DATA_FIELD_NUMBER: _ClassVar[int]
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    topic: OptionalTopic
    def __init__(
        self,
        data: _Optional[bytes] = ...,
        topic: _Optional[_Union[OptionalTopic, _Mapping]] = ...,
    ) -> None: ...

class SendRequest(_message.Message):
    __slots__ = ('peer_id', 'msg')
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    msg: Message
    def __init__(
        self,
        peer_id: _Optional[str] = ...,
        msg: _Optional[_Union[Message, _Mapping]] = ...,
    ) -> None: ...

class RecvRequest(_message.Message):
    __slots__ = ('peer_id', 'msg', 'seq')
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    SEQ_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    msg: Message
    seq: int
    def __init__(
        self,
        peer_id: _Optional[str] = ...,
        msg: _Optional[_Union[Message, _Mapping]] = ...,
        seq: _Optional[int] = ...,
    ) -> None: ...

class Response(_message.Message):
    __slots__ = ('data', 'error')
    DATA_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    error: str
    def __init__(
        self, data: _Optional[bytes] = ..., error: _Optional[str] = ...
    ) -> None: ...

class SendResponse(_message.Message):
    __slots__ = ('seq', 'response')
    SEQ_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    seq: int
    response: Response
    def __init__(
        self,
        seq: _Optional[int] = ...,
        response: _Optional[_Union[Response, _Mapping]] = ...,
    ) -> None: ...

class Peer(_message.Message):
    __slots__ = ('peer_id',)
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    def __init__(self, peer_id: _Optional[str] = ...) -> None: ...

class Peers(_message.Message):
    __slots__ = ('peers',)
    PEERS_FIELD_NUMBER: _ClassVar[int]
    peers: _containers.RepeatedCompositeFieldContainer[Peer]
    def __init__(
        self, peers: _Optional[_Iterable[_Union[Peer, _Mapping]]] = ...
    ) -> None: ...

class NeighbourEvent(_message.Message):
    __slots__ = ('init', 'discovered', 'lost')
    INIT_FIELD_NUMBER: _ClassVar[int]
    DISCOVERED_FIELD_NUMBER: _ClassVar[int]
    LOST_FIELD_NUMBER: _ClassVar[int]
    init: Peers
    discovered: Peer
    lost: Peer
    def __init__(
        self,
        init: _Optional[_Union[Peers, _Mapping]] = ...,
        discovered: _Optional[_Union[Peer, _Mapping]] = ...,
        lost: _Optional[_Union[Peer, _Mapping]] = ...,
    ) -> None: ...

class MeshTopologyEvent(_message.Message):
    __slots__ = ('peer', 'event')
    PEER_FIELD_NUMBER: _ClassVar[int]
    EVENT_FIELD_NUMBER: _ClassVar[int]
    peer: Peer
    event: NeighbourEvent
    def __init__(
        self,
        peer: _Optional[_Union[Peer, _Mapping]] = ...,
        event: _Optional[_Union[NeighbourEvent, _Mapping]] = ...,
    ) -> None: ...

class GossipSubMessageID(_message.Message):
    __slots__ = ('id',)
    ID_FIELD_NUMBER: _ClassVar[int]
    id: bytes
    def __init__(self, id: _Optional[bytes] = ...) -> None: ...

class GossipSubMessage(_message.Message):
    __slots__ = ('data', 'topic')
    DATA_FIELD_NUMBER: _ClassVar[int]
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    topic: Topic
    def __init__(
        self,
        data: _Optional[bytes] = ...,
        topic: _Optional[_Union[Topic, _Mapping]] = ...,
    ) -> None: ...

class GossipSubRecvMessage(_message.Message):
    __slots__ = ('peer_id', 'msg', 'msg_id')
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    MSG_FIELD_NUMBER: _ClassVar[int]
    MSG_ID_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    msg: GossipSubMessage
    msg_id: GossipSubMessageID
    def __init__(
        self,
        peer_id: _Optional[str] = ...,
        msg: _Optional[_Union[GossipSubMessage, _Mapping]] = ...,
        msg_id: _Optional[_Union[GossipSubMessageID, _Mapping]] = ...,
    ) -> None: ...

class DHTKey(_message.Message):
    __slots__ = ('topic', 'key')
    TOPIC_FIELD_NUMBER: _ClassVar[int]
    KEY_FIELD_NUMBER: _ClassVar[int]
    topic: Topic
    key: bytes
    def __init__(
        self,
        topic: _Optional[_Union[Topic, _Mapping]] = ...,
        key: _Optional[bytes] = ...,
    ) -> None: ...

class DHTPutRecord(_message.Message):
    __slots__ = ('key', 'value')
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: DHTKey
    value: bytes
    def __init__(
        self,
        key: _Optional[_Union[DHTKey, _Mapping]] = ...,
        value: _Optional[bytes] = ...,
    ) -> None: ...

class DHTGetRecord(_message.Message):
    __slots__ = ('key', 'value')
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: DHTKey
    value: bytes
    def __init__(
        self,
        key: _Optional[_Union[DHTKey, _Mapping]] = ...,
        value: _Optional[bytes] = ...,
    ) -> None: ...

class FilePath(_message.Message):
    __slots__ = ('path',)
    PATH_FIELD_NUMBER: _ClassVar[int]
    path: str
    def __init__(self, path: _Optional[str] = ...) -> None: ...

class CID(_message.Message):
    __slots__ = ('hash', 'id')
    HASH_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    hash: bytes
    id: ID
    def __init__(
        self, hash: _Optional[bytes] = ..., id: _Optional[_Union[ID, _Mapping]] = ...
    ) -> None: ...
