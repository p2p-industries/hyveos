# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: script.proto
# Protobuf Python Version: 5.26.1
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x0cscript.proto\x12\x06script\"5\n\x07Request\x12\x0f\n\x07peer_id\x18\x01 \x02(\t\x12\x0c\n\x04\x64\x61ta\x18\x02 \x02(\x0c\x12\x0b\n\x03seq\x18\x03 \x01(\x04\"\x18\n\x08Response\x12\x0c\n\x04\x64\x61ta\x18\x01 \x02(\x0c\"\x07\n\x05\x45mpty\"?\n\x0cSendResponse\x12\x0b\n\x03seq\x18\x01 \x02(\x04\x12\"\n\x08response\x18\x02 \x02(\x0b\x32\x10.script.Response\"\x1d\n\nDiscovered\x12\x0f\n\x07peer_id\x18\x01 \x02(\t2\x94\x01\n\x07ReqResp\x12+\n\x04Send\x12\x0f.script.Request\x1a\x10.script.Response\"\x00\x12*\n\x04Recv\x12\r.script.Empty\x1a\x0f.script.Request\"\x00\x30\x01\x12\x30\n\x07Respond\x12\x14.script.SendResponse\x1a\r.script.Empty\"\x00\x32>\n\tDiscovery\x12\x31\n\x08\x44iscover\x12\r.script.Empty\x1a\x12.script.Discovered\"\x00\x30\x01')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'script_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_REQUEST']._serialized_start=24
  _globals['_REQUEST']._serialized_end=77
  _globals['_RESPONSE']._serialized_start=79
  _globals['_RESPONSE']._serialized_end=103
  _globals['_EMPTY']._serialized_start=105
  _globals['_EMPTY']._serialized_end=112
  _globals['_SENDRESPONSE']._serialized_start=114
  _globals['_SENDRESPONSE']._serialized_end=177
  _globals['_DISCOVERED']._serialized_start=179
  _globals['_DISCOVERED']._serialized_end=208
  _globals['_REQRESP']._serialized_start=211
  _globals['_REQRESP']._serialized_end=359
  _globals['_DISCOVERY']._serialized_start=361
  _globals['_DISCOVERY']._serialized_end=423
# @@protoc_insertion_point(module_scope)
