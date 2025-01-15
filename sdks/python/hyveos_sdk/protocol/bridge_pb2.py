# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: bridge.proto
# Protobuf Python Version: 5.29.0
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    5,
    29,
    0,
    '',
    'bridge.proto'
)
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x0c\x62ridge.proto\x12\x06\x62ridge\"\x07\n\x05\x45mpty\"\x14\n\x04\x44\x61ta\x12\x0c\n\x04\x64\x61ta\x18\x01 \x02(\x0c\"*\n\x0cOptionalData\x12\x1a\n\x04\x64\x61ta\x18\x01 \x01(\x0b\x32\x0c.bridge.Data\"\x12\n\x02ID\x12\x0c\n\x04ulid\x18\x01 \x02(\t\"\x17\n\x04Peer\x12\x0f\n\x07peer_id\x18\x01 \x02(\t\"\x16\n\x05Topic\x12\r\n\x05topic\x18\x01 \x02(\t\"-\n\rOptionalTopic\x12\x1c\n\x05topic\x18\x01 \x01(\x0b\x32\r.bridge.Topic\"F\n\nTopicQuery\x12\x1e\n\x05topic\x18\x01 \x01(\x0b\x32\r.bridge.TopicH\x00\x12\x0f\n\x05regex\x18\x02 \x01(\tH\x00\x42\x07\n\x05query\"7\n\x12OptionalTopicQuery\x12!\n\x05query\x18\x01 \x01(\x0b\x32\x12.bridge.TopicQuery\"K\n\x07Message\x12\x1a\n\x04\x64\x61ta\x18\x01 \x02(\x0b\x32\x0c.bridge.Data\x12$\n\x05topic\x18\x02 \x02(\x0b\x32\x15.bridge.OptionalTopic\"G\n\x0bSendRequest\x12\x1a\n\x04peer\x18\x01 \x02(\x0b\x32\x0c.bridge.Peer\x12\x1c\n\x03msg\x18\x02 \x02(\x0b\x32\x0f.bridge.Message\"T\n\x0bRecvRequest\x12\x1a\n\x04peer\x18\x01 \x02(\x0b\x32\x0c.bridge.Peer\x12\x1c\n\x03msg\x18\x02 \x02(\x0b\x32\x0f.bridge.Message\x12\x0b\n\x03seq\x18\x03 \x02(\x04\"E\n\x08Response\x12\x1c\n\x04\x64\x61ta\x18\x01 \x01(\x0b\x32\x0c.bridge.DataH\x00\x12\x0f\n\x05\x65rror\x18\x02 \x01(\tH\x00\x42\n\n\x08response\"?\n\x0cSendResponse\x12\x0b\n\x03seq\x18\x01 \x02(\x04\x12\"\n\x08response\x18\x02 \x02(\x0b\x32\x10.bridge.Response\"$\n\x05Peers\x12\x1b\n\x05peers\x18\x01 \x03(\x0b\x32\x0c.bridge.Peer\"z\n\x0eNeighbourEvent\x12\x1d\n\x04init\x18\x01 \x01(\x0b\x32\r.bridge.PeersH\x00\x12\"\n\ndiscovered\x18\x02 \x01(\x0b\x32\x0c.bridge.PeerH\x00\x12\x1c\n\x04lost\x18\x03 \x01(\x0b\x32\x0c.bridge.PeerH\x00\x42\x07\n\x05\x65vent\"\x1d\n\x0fPubSubMessageID\x12\n\n\x02id\x18\x01 \x02(\x0c\"I\n\rPubSubMessage\x12\x1a\n\x04\x64\x61ta\x18\x01 \x02(\x0b\x32\x0c.bridge.Data\x12\x1c\n\x05topic\x18\x02 \x02(\x0b\x32\r.bridge.Topic\"\xa8\x01\n\x11PubSubRecvMessage\x12(\n\x12propagation_source\x18\x01 \x02(\x0b\x32\x0c.bridge.Peer\x12\x1c\n\x06source\x18\x02 \x01(\x0b\x32\x0c.bridge.Peer\x12\"\n\x03msg\x18\x03 \x02(\x0b\x32\x15.bridge.PubSubMessage\x12\'\n\x06msg_id\x18\x04 \x02(\x0b\x32\x17.bridge.PubSubMessageID\"3\n\x06\x44HTKey\x12\x1c\n\x05topic\x18\x01 \x02(\x0b\x32\r.bridge.Topic\x12\x0b\n\x03key\x18\x02 \x02(\x0c\"E\n\tDHTRecord\x12\x1b\n\x03key\x18\x01 \x02(\x0b\x32\x0e.bridge.DHTKey\x12\x1b\n\x05value\x18\x02 \x02(\x0b\x32\x0c.bridge.Data\"9\n\rLocalKVRecord\x12\x0b\n\x03key\x18\x01 \x02(\t\x12\x1b\n\x05value\x18\x02 \x02(\x0b\x32\x0c.bridge.Data\"\x19\n\nLocalKVKey\x12\x0b\n\x03key\x18\x01 \x02(\t\"\x18\n\x08\x46ilePath\x12\x0c\n\x04path\x18\x01 \x02(\t\"+\n\x03\x43ID\x12\x0c\n\x04hash\x18\x01 \x02(\x0c\x12\x16\n\x02id\x18\x02 \x02(\x0b\x32\n.bridge.ID\"O\n\rDownloadEvent\x12\x12\n\x08progress\x18\x01 \x01(\x04H\x00\x12!\n\x05ready\x18\x02 \x01(\x0b\x32\x10.bridge.FilePathH\x00\x42\x07\n\x05\x65vent\"V\n\x11MeshTopologyEvent\x12\x1a\n\x04peer\x18\x01 \x02(\x0b\x32\x0c.bridge.Peer\x12%\n\x05\x65vent\x18\x02 \x02(\x0b\x32\x16.bridge.NeighbourEvent\"i\n\x11RequestDebugEvent\x12\x16\n\x02id\x18\x01 \x02(\x0b\x32\n.bridge.ID\x12\x1e\n\x08receiver\x18\x02 \x02(\x0b\x32\x0c.bridge.Peer\x12\x1c\n\x03msg\x18\x03 \x02(\x0b\x32\x0f.bridge.Message\"T\n\x12ResponseDebugEvent\x12\x1a\n\x06req_id\x18\x01 \x02(\x0b\x32\n.bridge.ID\x12\"\n\x08response\x18\x02 \x02(\x0b\x32\x10.bridge.Response\"\xb9\x01\n\x11MessageDebugEvent\x12\x1c\n\x06sender\x18\x01 \x02(\x0b\x32\x0c.bridge.Peer\x12(\n\x03req\x18\x02 \x01(\x0b\x32\x19.bridge.RequestDebugEventH\x00\x12)\n\x03res\x18\x03 \x01(\x0b\x32\x1a.bridge.ResponseDebugEventH\x00\x12(\n\x07pub_sub\x18\x04 \x01(\x0b\x32\x15.bridge.PubSubMessageH\x00\x42\x07\n\x05\x65vent\"\x1b\n\x0b\x44ockerImage\x12\x0c\n\x04name\x18\x01 \x02(\t\">\n\tDockerApp\x12\"\n\x05image\x18\x01 \x02(\x0b\x32\x13.bridge.DockerImage\x12\r\n\x05ports\x18\x02 \x03(\r\"q\n\x10\x44\x65ployAppRequest\x12\x1e\n\x03\x61pp\x18\x01 \x02(\x0b\x32\x11.bridge.DockerApp\x12\r\n\x05local\x18\x02 \x02(\x08\x12\x1a\n\x04peer\x18\x03 \x01(\x0b\x32\x0c.bridge.Peer\x12\x12\n\npersistent\x18\x04 \x02(\x08\"4\n\x16ListRunningAppsRequest\x12\x1a\n\x04peer\x18\x01 \x01(\x0b\x32\x0c.bridge.Peer\"V\n\nRunningApp\x12\x16\n\x02id\x18\x01 \x02(\x0b\x32\n.bridge.ID\x12\"\n\x05image\x18\x02 \x02(\x0b\x32\x13.bridge.DockerImage\x12\x0c\n\x04name\x18\x03 \x01(\t\"/\n\x0bRunningApps\x12 \n\x04\x61pps\x18\x01 \x03(\x0b\x32\x12.bridge.RunningApp\"D\n\x0eStopAppRequest\x12\x16\n\x02id\x18\x01 \x02(\x0b\x32\n.bridge.ID\x12\x1a\n\x04peer\x18\x02 \x01(\x0b\x32\x0c.bridge.Peer2\xa9\x01\n\x07ReqResp\x12/\n\x04Send\x12\x13.bridge.SendRequest\x1a\x10.bridge.Response\"\x00\x12;\n\x04Recv\x12\x1a.bridge.OptionalTopicQuery\x1a\x13.bridge.RecvRequest\"\x00\x30\x01\x12\x30\n\x07Respond\x12\x14.bridge.SendResponse\x1a\r.bridge.Empty\"\x00\x32k\n\nNeighbours\x12\x36\n\tSubscribe\x12\r.bridge.Empty\x1a\x16.bridge.NeighbourEvent\"\x00\x30\x01\x12%\n\x03Get\x12\r.bridge.Empty\x1a\r.bridge.Peers\"\x00\x32\x80\x01\n\x06PubSub\x12\x39\n\tSubscribe\x12\r.bridge.Topic\x1a\x19.bridge.PubSubRecvMessage\"\x00\x30\x01\x12;\n\x07Publish\x12\x15.bridge.PubSubMessage\x1a\x17.bridge.PubSubMessageID\"\x00\x32\x9b\x01\n\x02KV\x12/\n\tPutRecord\x12\x11.bridge.DHTRecord\x1a\r.bridge.Empty\"\x00\x12\x33\n\tGetRecord\x12\x0e.bridge.DHTKey\x1a\x14.bridge.OptionalData\"\x00\x12/\n\x0cRemoveRecord\x12\x0e.bridge.DHTKey\x1a\r.bridge.Empty\"\x00\x32\x9b\x01\n\tDiscovery\x12*\n\x07Provide\x12\x0e.bridge.DHTKey\x1a\r.bridge.Empty\"\x00\x12\x30\n\x0cGetProviders\x12\x0e.bridge.DHTKey\x1a\x0c.bridge.Peer\"\x00\x30\x01\x12\x30\n\rStopProviding\x12\x0e.bridge.DHTKey\x1a\r.bridge.Empty\"\x00\x32r\n\x07LocalKV\x12\x34\n\x03Put\x12\x15.bridge.LocalKVRecord\x1a\x14.bridge.OptionalData\"\x00\x12\x31\n\x03Get\x12\x12.bridge.LocalKVKey\x1a\x14.bridge.OptionalData\"\x00\x32\x9d\x01\n\x0c\x46ileTransfer\x12*\n\x07Publish\x12\x10.bridge.FilePath\x1a\x0b.bridge.CID\"\x00\x12&\n\x03Get\x12\x0b.bridge.CID\x1a\x10.bridge.FilePath\"\x00\x12\x39\n\x0fGetWithProgress\x12\x0b.bridge.CID\x1a\x15.bridge.DownloadEvent\"\x00\x30\x01\x32\x91\x01\n\x05\x44\x65\x62ug\x12\x45\n\x15SubscribeMeshTopology\x12\r.bridge.Empty\x1a\x19.bridge.MeshTopologyEvent\"\x00\x30\x01\x12\x41\n\x11SubscribeMessages\x12\r.bridge.Empty\x1a\x19.bridge.MessageDebugEvent\"\x00\x30\x01\x32\xdb\x01\n\x04\x41pps\x12\x30\n\x06\x44\x65ploy\x12\x18.bridge.DeployAppRequest\x1a\n.bridge.ID\"\x00\x12\x44\n\x0bListRunning\x12\x1e.bridge.ListRunningAppsRequest\x1a\x13.bridge.RunningApps\"\x00\x12/\n\x04Stop\x12\x16.bridge.StopAppRequest\x1a\r.bridge.Empty\"\x00\x12*\n\x0bGetOwnAppId\x12\r.bridge.Empty\x1a\n.bridge.ID\"\x00\x32^\n\x07\x43ontrol\x12+\n\tHeartbeat\x12\r.bridge.Empty\x1a\r.bridge.Empty\"\x00\x12&\n\x05GetId\x12\r.bridge.Empty\x1a\x0c.bridge.Peer\"\x00')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'bridge_pb2', _globals)
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
  _globals['_PUBSUBMESSAGEID']._serialized_start=878
  _globals['_PUBSUBMESSAGEID']._serialized_end=907
  _globals['_PUBSUBMESSAGE']._serialized_start=909
  _globals['_PUBSUBMESSAGE']._serialized_end=982
  _globals['_PUBSUBRECVMESSAGE']._serialized_start=985
  _globals['_PUBSUBRECVMESSAGE']._serialized_end=1153
  _globals['_DHTKEY']._serialized_start=1155
  _globals['_DHTKEY']._serialized_end=1206
  _globals['_DHTRECORD']._serialized_start=1208
  _globals['_DHTRECORD']._serialized_end=1277
  _globals['_LOCALKVRECORD']._serialized_start=1279
  _globals['_LOCALKVRECORD']._serialized_end=1336
  _globals['_LOCALKVKEY']._serialized_start=1338
  _globals['_LOCALKVKEY']._serialized_end=1363
  _globals['_FILEPATH']._serialized_start=1365
  _globals['_FILEPATH']._serialized_end=1389
  _globals['_CID']._serialized_start=1391
  _globals['_CID']._serialized_end=1434
  _globals['_DOWNLOADEVENT']._serialized_start=1436
  _globals['_DOWNLOADEVENT']._serialized_end=1515
  _globals['_MESHTOPOLOGYEVENT']._serialized_start=1517
  _globals['_MESHTOPOLOGYEVENT']._serialized_end=1603
  _globals['_REQUESTDEBUGEVENT']._serialized_start=1605
  _globals['_REQUESTDEBUGEVENT']._serialized_end=1710
  _globals['_RESPONSEDEBUGEVENT']._serialized_start=1712
  _globals['_RESPONSEDEBUGEVENT']._serialized_end=1796
  _globals['_MESSAGEDEBUGEVENT']._serialized_start=1799
  _globals['_MESSAGEDEBUGEVENT']._serialized_end=1984
  _globals['_DOCKERIMAGE']._serialized_start=1986
  _globals['_DOCKERIMAGE']._serialized_end=2013
  _globals['_DOCKERAPP']._serialized_start=2015
  _globals['_DOCKERAPP']._serialized_end=2077
  _globals['_DEPLOYAPPREQUEST']._serialized_start=2079
  _globals['_DEPLOYAPPREQUEST']._serialized_end=2192
  _globals['_LISTRUNNINGAPPSREQUEST']._serialized_start=2194
  _globals['_LISTRUNNINGAPPSREQUEST']._serialized_end=2246
  _globals['_RUNNINGAPP']._serialized_start=2248
  _globals['_RUNNINGAPP']._serialized_end=2334
  _globals['_RUNNINGAPPS']._serialized_start=2336
  _globals['_RUNNINGAPPS']._serialized_end=2383
  _globals['_STOPAPPREQUEST']._serialized_start=2385
  _globals['_STOPAPPREQUEST']._serialized_end=2453
  _globals['_REQRESP']._serialized_start=2456
  _globals['_REQRESP']._serialized_end=2625
  _globals['_NEIGHBOURS']._serialized_start=2627
  _globals['_NEIGHBOURS']._serialized_end=2734
  _globals['_PUBSUB']._serialized_start=2737
  _globals['_PUBSUB']._serialized_end=2865
  _globals['_KV']._serialized_start=2868
  _globals['_KV']._serialized_end=3023
  _globals['_DISCOVERY']._serialized_start=3026
  _globals['_DISCOVERY']._serialized_end=3181
  _globals['_LOCALKV']._serialized_start=3183
  _globals['_LOCALKV']._serialized_end=3297
  _globals['_FILETRANSFER']._serialized_start=3300
  _globals['_FILETRANSFER']._serialized_end=3457
  _globals['_DEBUG']._serialized_start=3460
  _globals['_DEBUG']._serialized_end=3605
  _globals['_APPS']._serialized_start=3608
  _globals['_APPS']._serialized_end=3827
  _globals['_CONTROL']._serialized_start=3829
  _globals['_CONTROL']._serialized_end=3923
# @@protoc_insertion_point(module_scope)
