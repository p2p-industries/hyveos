# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: script.proto
# Protobuf Python Version: 5.27.2
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    5,
    27,
    2,
    '',
    'script.proto'
)
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x0cscript.proto\x12\x06script\"\x07\n\x05\x45mpty\"\x14\n\x04\x44\x61ta\x12\x0c\n\x04\x64\x61ta\x18\x01 \x02(\x0c\"*\n\x0cOptionalData\x12\x1a\n\x04\x64\x61ta\x18\x01 \x01(\x0b\x32\x0c.script.Data\"\x12\n\x02ID\x12\x0c\n\x04ulid\x18\x01 \x02(\t\"\x17\n\x04Peer\x12\x0f\n\x07peer_id\x18\x01 \x02(\t\"\x16\n\x05Topic\x12\r\n\x05topic\x18\x01 \x02(\t\"-\n\rOptionalTopic\x12\x1c\n\x05topic\x18\x01 \x01(\x0b\x32\r.script.Topic\"F\n\nTopicQuery\x12\x1e\n\x05topic\x18\x01 \x01(\x0b\x32\r.script.TopicH\x00\x12\x0f\n\x05regex\x18\x02 \x01(\tH\x00\x42\x07\n\x05query\"7\n\x12OptionalTopicQuery\x12!\n\x05query\x18\x01 \x01(\x0b\x32\x12.script.TopicQuery\"K\n\x07Message\x12\x1a\n\x04\x64\x61ta\x18\x01 \x02(\x0b\x32\x0c.script.Data\x12$\n\x05topic\x18\x02 \x02(\x0b\x32\x15.script.OptionalTopic\"G\n\x0bSendRequest\x12\x1a\n\x04peer\x18\x01 \x02(\x0b\x32\x0c.script.Peer\x12\x1c\n\x03msg\x18\x02 \x02(\x0b\x32\x0f.script.Message\"T\n\x0bRecvRequest\x12\x1a\n\x04peer\x18\x01 \x02(\x0b\x32\x0c.script.Peer\x12\x1c\n\x03msg\x18\x02 \x02(\x0b\x32\x0f.script.Message\x12\x0b\n\x03seq\x18\x03 \x02(\x04\"E\n\x08Response\x12\x1c\n\x04\x64\x61ta\x18\x01 \x01(\x0b\x32\x0c.script.DataH\x00\x12\x0f\n\x05\x65rror\x18\x02 \x01(\tH\x00\x42\n\n\x08response\"?\n\x0cSendResponse\x12\x0b\n\x03seq\x18\x01 \x02(\x04\x12\"\n\x08response\x18\x02 \x02(\x0b\x32\x10.script.Response\"$\n\x05Peers\x12\x1b\n\x05peers\x18\x01 \x03(\x0b\x32\x0c.script.Peer\"z\n\x0eNeighbourEvent\x12\x1d\n\x04init\x18\x01 \x01(\x0b\x32\r.script.PeersH\x00\x12\"\n\ndiscovered\x18\x02 \x01(\x0b\x32\x0c.script.PeerH\x00\x12\x1c\n\x04lost\x18\x03 \x01(\x0b\x32\x0c.script.PeerH\x00\x42\x07\n\x05\x65vent\"V\n\x11MeshTopologyEvent\x12\x1a\n\x04peer\x18\x01 \x02(\x0b\x32\x0c.script.Peer\x12%\n\x05\x65vent\x18\x02 \x02(\x0b\x32\x16.script.NeighbourEvent\" \n\x12GossipSubMessageID\x12\n\n\x02id\x18\x01 \x02(\x0c\"L\n\x10GossipSubMessage\x12\x1a\n\x04\x64\x61ta\x18\x01 \x02(\x0b\x32\x0c.script.Data\x12\x1c\n\x05topic\x18\x02 \x02(\x0b\x32\r.script.Topic\"\xb1\x01\n\x14GossipSubRecvMessage\x12(\n\x12propagation_source\x18\x01 \x02(\x0b\x32\x0c.script.Peer\x12\x1c\n\x06source\x18\x02 \x01(\x0b\x32\x0c.script.Peer\x12%\n\x03msg\x18\x03 \x02(\x0b\x32\x18.script.GossipSubMessage\x12*\n\x06msg_id\x18\x04 \x02(\x0b\x32\x1a.script.GossipSubMessageID\"3\n\x06\x44HTKey\x12\x1c\n\x05topic\x18\x01 \x02(\x0b\x32\r.script.Topic\x12\x0b\n\x03key\x18\x02 \x02(\x0c\"E\n\tDHTRecord\x12\x1b\n\x03key\x18\x01 \x02(\x0b\x32\x0e.script.DHTKey\x12\x1b\n\x05value\x18\x02 \x02(\x0b\x32\x0c.script.Data\"4\n\x08\x44\x42Record\x12\x0b\n\x03key\x18\x01 \x02(\t\x12\x1b\n\x05value\x18\x02 \x02(\x0b\x32\x0c.script.Data\"\x14\n\x05\x44\x42Key\x12\x0b\n\x03key\x18\x01 \x02(\t\"\x18\n\x08\x46ilePath\x12\x0c\n\x04path\x18\x01 \x02(\t\"+\n\x03\x43ID\x12\x0c\n\x04hash\x18\x01 \x02(\x0c\x12\x16\n\x02id\x18\x02 \x02(\x0b\x32\n.script.ID\"\x1b\n\x0b\x44ockerImage\x12\x0c\n\x04name\x18\x01 \x02(\t\"A\n\x0c\x44ockerScript\x12\"\n\x05image\x18\x01 \x02(\x0b\x32\x13.script.DockerImage\x12\r\n\x05ports\x18\x02 \x03(\r\"z\n\x13\x44\x65ployScriptRequest\x12$\n\x06script\x18\x01 \x02(\x0b\x32\x14.script.DockerScript\x12\r\n\x05local\x18\x02 \x02(\x08\x12\x1a\n\x04peer\x18\x03 \x01(\x0b\x32\x0c.script.Peer\x12\x12\n\npersistent\x18\x04 \x02(\x08\"7\n\x19ListRunningScriptsRequest\x12\x1a\n\x04peer\x18\x01 \x01(\x0b\x32\x0c.script.Peer\"Y\n\rRunningScript\x12\x16\n\x02id\x18\x01 \x02(\x0b\x32\n.script.ID\x12\"\n\x05image\x18\x02 \x02(\x0b\x32\x13.script.DockerImage\x12\x0c\n\x04name\x18\x03 \x01(\t\"8\n\x0eRunningScripts\x12&\n\x07scripts\x18\x01 \x03(\x0b\x32\x15.script.RunningScript\"G\n\x11StopScriptRequest\x12\x16\n\x02id\x18\x01 \x02(\x0b\x32\n.script.ID\x12\x1a\n\x04peer\x18\x02 \x01(\x0b\x32\x0c.script.Peer2\xa9\x01\n\x07ReqResp\x12/\n\x04Send\x12\x13.script.SendRequest\x1a\x10.script.Response\"\x00\x12;\n\x04Recv\x12\x1a.script.OptionalTopicQuery\x1a\x13.script.RecvRequest\"\x00\x30\x01\x12\x30\n\x07Respond\x12\x14.script.SendResponse\x1a\r.script.Empty\"\x00\x32t\n\tDiscovery\x12<\n\x0fSubscribeEvents\x12\r.script.Empty\x1a\x16.script.NeighbourEvent\"\x00\x30\x01\x12)\n\x08GetOwnId\x12\r.script.Empty\x1a\x0c.script.Peer\"\x00\x32\x8c\x01\n\tGossipSub\x12<\n\tSubscribe\x12\r.script.Topic\x1a\x1c.script.GossipSubRecvMessage\"\x00\x30\x01\x12\x41\n\x07Publish\x12\x18.script.GossipSubMessage\x1a\x1a.script.GossipSubMessageID\"\x00\x32\xc9\x01\n\x03\x44HT\x12/\n\tPutRecord\x12\x11.script.DHTRecord\x1a\r.script.Empty\"\x00\x12\x33\n\tGetRecord\x12\x0e.script.DHTKey\x1a\x14.script.OptionalData\"\x00\x12*\n\x07Provide\x12\x0e.script.DHTKey\x1a\r.script.Empty\"\x00\x12\x30\n\x0cGetProviders\x12\x0e.script.DHTKey\x1a\x0c.script.Peer\"\x00\x30\x01\x32\x63\n\x02\x44\x42\x12/\n\x03Put\x12\x10.script.DBRecord\x1a\x14.script.OptionalData\"\x00\x12,\n\x03Get\x12\r.script.DBKey\x1a\x14.script.OptionalData\"\x00\x32j\n\x0c\x46ileTransfer\x12.\n\x0bPublishFile\x12\x10.script.FilePath\x1a\x0b.script.CID\"\x00\x12*\n\x07GetFile\x12\x0b.script.CID\x1a\x10.script.FilePath\"\x00\x32N\n\x05\x44\x65\x62ug\x12\x45\n\x15SubscribeMeshTopology\x12\r.script.Empty\x1a\x19.script.MeshTopologyEvent\"\x00\x30\x01\x32\xfc\x01\n\tScripting\x12\x39\n\x0c\x44\x65ployScript\x12\x1b.script.DeployScriptRequest\x1a\n.script.ID\"\x00\x12Q\n\x12ListRunningScripts\x12!.script.ListRunningScriptsRequest\x1a\x16.script.RunningScripts\"\x00\x12\x38\n\nStopScript\x12\x19.script.StopScriptRequest\x1a\r.script.Empty\"\x00\x12\'\n\x08GetOwnId\x12\r.script.Empty\x1a\n.script.ID\"\x00')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'script_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_EMPTY']._serialized_start=24
  _globals['_EMPTY']._serialized_end=31
  _globals['_DATA']._serialized_start=33
  _globals['_DATA']._serialized_end=53
  _globals['_OPTIONALDATA']._serialized_start=55
  _globals['_OPTIONALDATA']._serialized_end=97
  _globals['_ID']._serialized_start=99
  _globals['_ID']._serialized_end=117
  _globals['_PEER']._serialized_start=119
  _globals['_PEER']._serialized_end=142
  _globals['_TOPIC']._serialized_start=144
  _globals['_TOPIC']._serialized_end=166
  _globals['_OPTIONALTOPIC']._serialized_start=168
  _globals['_OPTIONALTOPIC']._serialized_end=213
  _globals['_TOPICQUERY']._serialized_start=215
  _globals['_TOPICQUERY']._serialized_end=285
  _globals['_OPTIONALTOPICQUERY']._serialized_start=287
  _globals['_OPTIONALTOPICQUERY']._serialized_end=342
  _globals['_MESSAGE']._serialized_start=344
  _globals['_MESSAGE']._serialized_end=419
  _globals['_SENDREQUEST']._serialized_start=421
  _globals['_SENDREQUEST']._serialized_end=492
  _globals['_RECVREQUEST']._serialized_start=494
  _globals['_RECVREQUEST']._serialized_end=578
  _globals['_RESPONSE']._serialized_start=580
  _globals['_RESPONSE']._serialized_end=649
  _globals['_SENDRESPONSE']._serialized_start=651
  _globals['_SENDRESPONSE']._serialized_end=714
  _globals['_PEERS']._serialized_start=716
  _globals['_PEERS']._serialized_end=752
  _globals['_NEIGHBOUREVENT']._serialized_start=754
  _globals['_NEIGHBOUREVENT']._serialized_end=876
  _globals['_MESHTOPOLOGYEVENT']._serialized_start=878
  _globals['_MESHTOPOLOGYEVENT']._serialized_end=964
  _globals['_GOSSIPSUBMESSAGEID']._serialized_start=966
  _globals['_GOSSIPSUBMESSAGEID']._serialized_end=998
  _globals['_GOSSIPSUBMESSAGE']._serialized_start=1000
  _globals['_GOSSIPSUBMESSAGE']._serialized_end=1076
  _globals['_GOSSIPSUBRECVMESSAGE']._serialized_start=1079
  _globals['_GOSSIPSUBRECVMESSAGE']._serialized_end=1256
  _globals['_DHTKEY']._serialized_start=1258
  _globals['_DHTKEY']._serialized_end=1309
  _globals['_DHTRECORD']._serialized_start=1311
  _globals['_DHTRECORD']._serialized_end=1380
  _globals['_DBRECORD']._serialized_start=1382
  _globals['_DBRECORD']._serialized_end=1434
  _globals['_DBKEY']._serialized_start=1436
  _globals['_DBKEY']._serialized_end=1456
  _globals['_FILEPATH']._serialized_start=1458
  _globals['_FILEPATH']._serialized_end=1482
  _globals['_CID']._serialized_start=1484
  _globals['_CID']._serialized_end=1527
  _globals['_DOCKERIMAGE']._serialized_start=1529
  _globals['_DOCKERIMAGE']._serialized_end=1556
  _globals['_DOCKERSCRIPT']._serialized_start=1558
  _globals['_DOCKERSCRIPT']._serialized_end=1623
  _globals['_DEPLOYSCRIPTREQUEST']._serialized_start=1625
  _globals['_DEPLOYSCRIPTREQUEST']._serialized_end=1747
  _globals['_LISTRUNNINGSCRIPTSREQUEST']._serialized_start=1749
  _globals['_LISTRUNNINGSCRIPTSREQUEST']._serialized_end=1804
  _globals['_RUNNINGSCRIPT']._serialized_start=1806
  _globals['_RUNNINGSCRIPT']._serialized_end=1895
  _globals['_RUNNINGSCRIPTS']._serialized_start=1897
  _globals['_RUNNINGSCRIPTS']._serialized_end=1953
  _globals['_STOPSCRIPTREQUEST']._serialized_start=1955
  _globals['_STOPSCRIPTREQUEST']._serialized_end=2026
  _globals['_REQRESP']._serialized_start=2029
  _globals['_REQRESP']._serialized_end=2198
  _globals['_DISCOVERY']._serialized_start=2200
  _globals['_DISCOVERY']._serialized_end=2316
  _globals['_GOSSIPSUB']._serialized_start=2319
  _globals['_GOSSIPSUB']._serialized_end=2459
  _globals['_DHT']._serialized_start=2462
  _globals['_DHT']._serialized_end=2663
  _globals['_DB']._serialized_start=2665
  _globals['_DB']._serialized_end=2764
  _globals['_FILETRANSFER']._serialized_start=2766
  _globals['_FILETRANSFER']._serialized_end=2872
  _globals['_DEBUG']._serialized_start=2874
  _globals['_DEBUG']._serialized_end=2952
  _globals['_SCRIPTING']._serialized_start=2955
  _globals['_SCRIPTING']._serialized_end=3207
# @@protoc_insertion_point(module_scope)
