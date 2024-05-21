from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Request(_message.Message):
    __slots__ = ("peer_id", "data", "seq")
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    SEQ_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    data: bytes
    seq: int
    def __init__(self, peer_id: _Optional[str] = ..., data: _Optional[bytes] = ..., seq: _Optional[int] = ...) -> None: ...

class Response(_message.Message):
    __slots__ = ("data",)
    DATA_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    def __init__(self, data: _Optional[bytes] = ...) -> None: ...

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SendResponse(_message.Message):
    __slots__ = ("seq", "response")
    SEQ_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    seq: int
    response: Response
    def __init__(self, seq: _Optional[int] = ..., response: _Optional[_Union[Response, _Mapping]] = ...) -> None: ...

class Discovered(_message.Message):
    __slots__ = ("peer_id",)
    PEER_ID_FIELD_NUMBER: _ClassVar[int]
    peer_id: str
    def __init__(self, peer_id: _Optional[str] = ...) -> None: ...
