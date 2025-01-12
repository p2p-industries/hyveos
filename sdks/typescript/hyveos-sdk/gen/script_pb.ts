// Put the basic example for grpc using proto2 here

// @generated by protoc-gen-es v2.2.3 with parameter "target=ts"
// @generated from file script.proto (package script, syntax proto2)
/* eslint-disable */

import type { GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv1";
import { fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv1";
import type { Message as Message$1 } from "@bufbuild/protobuf";

/**
 * Describes the file script.proto.
 */
export const file_script: GenFile = /*@__PURE__*/
  fileDesc("CgxzY3JpcHQucHJvdG8SBnNjcmlwdCIHCgVFbXB0eSIUCgREYXRhEgwKBGRhdGEYASACKAwiKgoMT3B0aW9uYWxEYXRhEhoKBGRhdGEYASABKAsyDC5zY3JpcHQuRGF0YSISCgJJRBIMCgR1bGlkGAEgAigJIhcKBFBlZXISDwoHcGVlcl9pZBgBIAIoCSIWCgVUb3BpYxINCgV0b3BpYxgBIAIoCSItCg1PcHRpb25hbFRvcGljEhwKBXRvcGljGAEgASgLMg0uc2NyaXB0LlRvcGljIkYKClRvcGljUXVlcnkSHgoFdG9waWMYASABKAsyDS5zY3JpcHQuVG9waWNIABIPCgVyZWdleBgCIAEoCUgAQgcKBXF1ZXJ5IjcKEk9wdGlvbmFsVG9waWNRdWVyeRIhCgVxdWVyeRgBIAEoCzISLnNjcmlwdC5Ub3BpY1F1ZXJ5IksKB01lc3NhZ2USGgoEZGF0YRgBIAIoCzIMLnNjcmlwdC5EYXRhEiQKBXRvcGljGAIgAigLMhUuc2NyaXB0Lk9wdGlvbmFsVG9waWMiRwoLU2VuZFJlcXVlc3QSGgoEcGVlchgBIAIoCzIMLnNjcmlwdC5QZWVyEhwKA21zZxgCIAIoCzIPLnNjcmlwdC5NZXNzYWdlIlQKC1JlY3ZSZXF1ZXN0EhoKBHBlZXIYASACKAsyDC5zY3JpcHQuUGVlchIcCgNtc2cYAiACKAsyDy5zY3JpcHQuTWVzc2FnZRILCgNzZXEYAyACKAQiRQoIUmVzcG9uc2USHAoEZGF0YRgBIAEoCzIMLnNjcmlwdC5EYXRhSAASDwoFZXJyb3IYAiABKAlIAEIKCghyZXNwb25zZSI/CgxTZW5kUmVzcG9uc2USCwoDc2VxGAEgAigEEiIKCHJlc3BvbnNlGAIgAigLMhAuc2NyaXB0LlJlc3BvbnNlIiQKBVBlZXJzEhsKBXBlZXJzGAEgAygLMgwuc2NyaXB0LlBlZXIiegoOTmVpZ2hib3VyRXZlbnQSHQoEaW5pdBgBIAEoCzINLnNjcmlwdC5QZWVyc0gAEiIKCmRpc2NvdmVyZWQYAiABKAsyDC5zY3JpcHQuUGVlckgAEhwKBGxvc3QYAyABKAsyDC5zY3JpcHQuUGVlckgAQgcKBWV2ZW50IiAKEkdvc3NpcFN1Yk1lc3NhZ2VJRBIKCgJpZBgBIAIoDCJMChBHb3NzaXBTdWJNZXNzYWdlEhoKBGRhdGEYASACKAsyDC5zY3JpcHQuRGF0YRIcCgV0b3BpYxgCIAIoCzINLnNjcmlwdC5Ub3BpYyKxAQoUR29zc2lwU3ViUmVjdk1lc3NhZ2USKAoScHJvcGFnYXRpb25fc291cmNlGAEgAigLMgwuc2NyaXB0LlBlZXISHAoGc291cmNlGAIgASgLMgwuc2NyaXB0LlBlZXISJQoDbXNnGAMgAigLMhguc2NyaXB0Lkdvc3NpcFN1Yk1lc3NhZ2USKgoGbXNnX2lkGAQgAigLMhouc2NyaXB0Lkdvc3NpcFN1Yk1lc3NhZ2VJRCIzCgZESFRLZXkSHAoFdG9waWMYASACKAsyDS5zY3JpcHQuVG9waWMSCwoDa2V5GAIgAigMIkUKCURIVFJlY29yZBIbCgNrZXkYASACKAsyDi5zY3JpcHQuREhUS2V5EhsKBXZhbHVlGAIgAigLMgwuc2NyaXB0LkRhdGEiNAoIREJSZWNvcmQSCwoDa2V5GAEgAigJEhsKBXZhbHVlGAIgAigLMgwuc2NyaXB0LkRhdGEiFAoFREJLZXkSCwoDa2V5GAEgAigJIhgKCEZpbGVQYXRoEgwKBHBhdGgYASACKAkiKwoDQ0lEEgwKBGhhc2gYASACKAwSFgoCaWQYAiACKAsyCi5zY3JpcHQuSUQiTwoNRG93bmxvYWRFdmVudBISCghwcm9ncmVzcxgBIAEoBEgAEiEKBXJlYWR5GAIgASgLMhAuc2NyaXB0LkZpbGVQYXRoSABCBwoFZXZlbnQiVgoRTWVzaFRvcG9sb2d5RXZlbnQSGgoEcGVlchgBIAIoCzIMLnNjcmlwdC5QZWVyEiUKBWV2ZW50GAIgAigLMhYuc2NyaXB0Lk5laWdoYm91ckV2ZW50ImkKEVJlcXVlc3REZWJ1Z0V2ZW50EhYKAmlkGAEgAigLMgouc2NyaXB0LklEEh4KCHJlY2VpdmVyGAIgAigLMgwuc2NyaXB0LlBlZXISHAoDbXNnGAMgAigLMg8uc2NyaXB0Lk1lc3NhZ2UiVAoSUmVzcG9uc2VEZWJ1Z0V2ZW50EhoKBnJlcV9pZBgBIAIoCzIKLnNjcmlwdC5JRBIiCghyZXNwb25zZRgCIAIoCzIQLnNjcmlwdC5SZXNwb25zZSK4AQoRTWVzc2FnZURlYnVnRXZlbnQSHAoGc2VuZGVyGAEgAigLMgwuc2NyaXB0LlBlZXISKAoDcmVxGAIgASgLMhkuc2NyaXB0LlJlcXVlc3REZWJ1Z0V2ZW50SAASKQoDcmVzGAMgASgLMhouc2NyaXB0LlJlc3BvbnNlRGVidWdFdmVudEgAEicKA2dvcxgEIAEoCzIYLnNjcmlwdC5Hb3NzaXBTdWJNZXNzYWdlSABCBwoFZXZlbnQiGwoLRG9ja2VySW1hZ2USDAoEbmFtZRgBIAIoCSJBCgxEb2NrZXJTY3JpcHQSIgoFaW1hZ2UYASACKAsyEy5zY3JpcHQuRG9ja2VySW1hZ2USDQoFcG9ydHMYAiADKA0iegoTRGVwbG95U2NyaXB0UmVxdWVzdBIkCgZzY3JpcHQYASACKAsyFC5zY3JpcHQuRG9ja2VyU2NyaXB0Eg0KBWxvY2FsGAIgAigIEhoKBHBlZXIYAyABKAsyDC5zY3JpcHQuUGVlchISCgpwZXJzaXN0ZW50GAQgAigIIjcKGUxpc3RSdW5uaW5nU2NyaXB0c1JlcXVlc3QSGgoEcGVlchgBIAEoCzIMLnNjcmlwdC5QZWVyIlkKDVJ1bm5pbmdTY3JpcHQSFgoCaWQYASACKAsyCi5zY3JpcHQuSUQSIgoFaW1hZ2UYAiACKAsyEy5zY3JpcHQuRG9ja2VySW1hZ2USDAoEbmFtZRgDIAEoCSI4Cg5SdW5uaW5nU2NyaXB0cxImCgdzY3JpcHRzGAEgAygLMhUuc2NyaXB0LlJ1bm5pbmdTY3JpcHQiRwoRU3RvcFNjcmlwdFJlcXVlc3QSFgoCaWQYASACKAsyCi5zY3JpcHQuSUQSGgoEcGVlchgCIAEoCzIMLnNjcmlwdC5QZWVyMqkBCgdSZXFSZXNwEi8KBFNlbmQSEy5zY3JpcHQuU2VuZFJlcXVlc3QaEC5zY3JpcHQuUmVzcG9uc2UiABI7CgRSZWN2Ehouc2NyaXB0Lk9wdGlvbmFsVG9waWNRdWVyeRoTLnNjcmlwdC5SZWN2UmVxdWVzdCIAMAESMAoHUmVzcG9uZBIULnNjcmlwdC5TZW5kUmVzcG9uc2UaDS5zY3JpcHQuRW1wdHkiADJ0CglEaXNjb3ZlcnkSPAoPU3Vic2NyaWJlRXZlbnRzEg0uc2NyaXB0LkVtcHR5GhYuc2NyaXB0Lk5laWdoYm91ckV2ZW50IgAwARIpCghHZXRPd25JZBINLnNjcmlwdC5FbXB0eRoMLnNjcmlwdC5QZWVyIgAyjAEKCUdvc3NpcFN1YhI8CglTdWJzY3JpYmUSDS5zY3JpcHQuVG9waWMaHC5zY3JpcHQuR29zc2lwU3ViUmVjdk1lc3NhZ2UiADABEkEKB1B1Ymxpc2gSGC5zY3JpcHQuR29zc2lwU3ViTWVzc2FnZRoaLnNjcmlwdC5Hb3NzaXBTdWJNZXNzYWdlSUQiADKsAgoDREhUEi8KCVB1dFJlY29yZBIRLnNjcmlwdC5ESFRSZWNvcmQaDS5zY3JpcHQuRW1wdHkiABIzCglHZXRSZWNvcmQSDi5zY3JpcHQuREhUS2V5GhQuc2NyaXB0Lk9wdGlvbmFsRGF0YSIAEi8KDFJlbW92ZVJlY29yZBIOLnNjcmlwdC5ESFRLZXkaDS5zY3JpcHQuRW1wdHkiABIqCgdQcm92aWRlEg4uc2NyaXB0LkRIVEtleRoNLnNjcmlwdC5FbXB0eSIAEjAKDEdldFByb3ZpZGVycxIOLnNjcmlwdC5ESFRLZXkaDC5zY3JpcHQuUGVlciIAMAESMAoNU3RvcFByb3ZpZGluZxIOLnNjcmlwdC5ESFRLZXkaDS5zY3JpcHQuRW1wdHkiADJjCgJEQhIvCgNQdXQSEC5zY3JpcHQuREJSZWNvcmQaFC5zY3JpcHQuT3B0aW9uYWxEYXRhIgASLAoDR2V0Eg0uc2NyaXB0LkRCS2V5GhQuc2NyaXB0Lk9wdGlvbmFsRGF0YSIAMqkBCgxGaWxlVHJhbnNmZXISLgoLUHVibGlzaEZpbGUSEC5zY3JpcHQuRmlsZVBhdGgaCy5zY3JpcHQuQ0lEIgASKgoHR2V0RmlsZRILLnNjcmlwdC5DSUQaEC5zY3JpcHQuRmlsZVBhdGgiABI9ChNHZXRGaWxlV2l0aFByb2dyZXNzEgsuc2NyaXB0LkNJRBoVLnNjcmlwdC5Eb3dubG9hZEV2ZW50IgAwATKRAQoFRGVidWcSRQoVU3Vic2NyaWJlTWVzaFRvcG9sb2d5Eg0uc2NyaXB0LkVtcHR5Ghkuc2NyaXB0Lk1lc2hUb3BvbG9neUV2ZW50IgAwARJBChFTdWJzY3JpYmVNZXNzYWdlcxINLnNjcmlwdC5FbXB0eRoZLnNjcmlwdC5NZXNzYWdlRGVidWdFdmVudCIAMAEy/AEKCVNjcmlwdGluZxI5CgxEZXBsb3lTY3JpcHQSGy5zY3JpcHQuRGVwbG95U2NyaXB0UmVxdWVzdBoKLnNjcmlwdC5JRCIAElEKEkxpc3RSdW5uaW5nU2NyaXB0cxIhLnNjcmlwdC5MaXN0UnVubmluZ1NjcmlwdHNSZXF1ZXN0GhYuc2NyaXB0LlJ1bm5pbmdTY3JpcHRzIgASOAoKU3RvcFNjcmlwdBIZLnNjcmlwdC5TdG9wU2NyaXB0UmVxdWVzdBoNLnNjcmlwdC5FbXB0eSIAEicKCEdldE93bklkEg0uc2NyaXB0LkVtcHR5Ggouc2NyaXB0LklEIgA");

/**
 * @generated from message script.Empty
 */
export type Empty = Message$1<"script.Empty"> & {
};

/**
 * Describes the message script.Empty.
 * Use `create(EmptySchema)` to create a new message.
 */
export const EmptySchema: GenMessage<Empty> = /*@__PURE__*/
  messageDesc(file_script, 0);

/**
 * @generated from message script.Data
 */
export type Data = Message$1<"script.Data"> & {
  /**
   * @generated from field: required bytes data = 1;
   */
  data: Uint8Array;
};

/**
 * Describes the message script.Data.
 * Use `create(DataSchema)` to create a new message.
 */
export const DataSchema: GenMessage<Data> = /*@__PURE__*/
  messageDesc(file_script, 1);

/**
 * @generated from message script.OptionalData
 */
export type OptionalData = Message$1<"script.OptionalData"> & {
  /**
   * @generated from field: optional script.Data data = 1;
   */
  data?: Data;
};

/**
 * Describes the message script.OptionalData.
 * Use `create(OptionalDataSchema)` to create a new message.
 */
export const OptionalDataSchema: GenMessage<OptionalData> = /*@__PURE__*/
  messageDesc(file_script, 2);

/**
 * Unique identifier in the form of a stringified ULID
 *
 * @generated from message script.ID
 */
export type ID = Message$1<"script.ID"> & {
  /**
   * @generated from field: required string ulid = 1;
   */
  ulid: string;
};

/**
 * Describes the message script.ID.
 * Use `create(IDSchema)` to create a new message.
 */
export const IDSchema: GenMessage<ID> = /*@__PURE__*/
  messageDesc(file_script, 3);

/**
 * A peer in the network
 *
 * @generated from message script.Peer
 */
export type Peer = Message$1<"script.Peer"> & {
  /**
   * @generated from field: required string peer_id = 1;
   */
  peerId: string;
};

/**
 * Describes the message script.Peer.
 * Use `create(PeerSchema)` to create a new message.
 */
export const PeerSchema: GenMessage<Peer> = /*@__PURE__*/
  messageDesc(file_script, 4);

/**
 * The topic of a message
 *
 * @generated from message script.Topic
 */
export type Topic = Message$1<"script.Topic"> & {
  /**
   * @generated from field: required string topic = 1;
   */
  topic: string;
};

/**
 * Describes the message script.Topic.
 * Use `create(TopicSchema)` to create a new message.
 */
export const TopicSchema: GenMessage<Topic> = /*@__PURE__*/
  messageDesc(file_script, 5);

/**
 * An optional topic for a message
 *
 * @generated from message script.OptionalTopic
 */
export type OptionalTopic = Message$1<"script.OptionalTopic"> & {
  /**
   * @generated from field: optional script.Topic topic = 1;
   */
  topic?: Topic;
};

/**
 * Describes the message script.OptionalTopic.
 * Use `create(OptionalTopicSchema)` to create a new message.
 */
export const OptionalTopicSchema: GenMessage<OptionalTopic> = /*@__PURE__*/
  messageDesc(file_script, 6);

/**
 * A query for a topic that can be a regex
 *
 * @generated from message script.TopicQuery
 */
export type TopicQuery = Message$1<"script.TopicQuery"> & {
  /**
   * @generated from oneof script.TopicQuery.query
   */
  query: {
    /**
     * @generated from field: script.Topic topic = 1;
     */
    value: Topic;
    case: "topic";
  } | {
    /**
     * @generated from field: string regex = 2;
     */
    value: string;
    case: "regex";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message script.TopicQuery.
 * Use `create(TopicQuerySchema)` to create a new message.
 */
export const TopicQuerySchema: GenMessage<TopicQuery> = /*@__PURE__*/
  messageDesc(file_script, 7);

/**
 * An optional query for a topic
 *
 * @generated from message script.OptionalTopicQuery
 */
export type OptionalTopicQuery = Message$1<"script.OptionalTopicQuery"> & {
  /**
   * @generated from field: optional script.TopicQuery query = 1;
   */
  query?: TopicQuery;
};

/**
 * Describes the message script.OptionalTopicQuery.
 * Use `create(OptionalTopicQuerySchema)` to create a new message.
 */
export const OptionalTopicQuerySchema: GenMessage<OptionalTopicQuery> = /*@__PURE__*/
  messageDesc(file_script, 8);

/**
 * A message with an optional topic
 *
 * @generated from message script.Message
 */
export type Message = Message$1<"script.Message"> & {
  /**
   * @generated from field: required script.Data data = 1;
   */
  data?: Data;

  /**
   * @generated from field: required script.OptionalTopic topic = 2;
   */
  topic?: OptionalTopic;
};

/**
 * Describes the message script.Message.
 * Use `create(MessageSchema)` to create a new message.
 */
export const MessageSchema: GenMessage<Message> = /*@__PURE__*/
  messageDesc(file_script, 9);

/**
 * A request to a peer with an optional topic
 *
 * @generated from message script.SendRequest
 */
export type SendRequest = Message$1<"script.SendRequest"> & {
  /**
   * @generated from field: required script.Peer peer = 1;
   */
  peer?: Peer;

  /**
   * @generated from field: required script.Message msg = 2;
   */
  msg?: Message;
};

/**
 * Describes the message script.SendRequest.
 * Use `create(SendRequestSchema)` to create a new message.
 */
export const SendRequestSchema: GenMessage<SendRequest> = /*@__PURE__*/
  messageDesc(file_script, 10);

/**
 * A request from a peer with an optional topic
 *
 * @generated from message script.RecvRequest
 */
export type RecvRequest = Message$1<"script.RecvRequest"> & {
  /**
   * @generated from field: required script.Peer peer = 1;
   */
  peer?: Peer;

  /**
   * @generated from field: required script.Message msg = 2;
   */
  msg?: Message;

  /**
   * Sequence number for request-response matching
   *
   * @generated from field: required uint64 seq = 3;
   */
  seq: bigint;
};

/**
 * Describes the message script.RecvRequest.
 * Use `create(RecvRequestSchema)` to create a new message.
 */
export const RecvRequestSchema: GenMessage<RecvRequest> = /*@__PURE__*/
  messageDesc(file_script, 11);

/**
 * A received response to a request or an error
 *
 * @generated from message script.Response
 */
export type Response = Message$1<"script.Response"> & {
  /**
   * @generated from oneof script.Response.response
   */
  response: {
    /**
     * @generated from field: script.Data data = 1;
     */
    value: Data;
    case: "data";
  } | {
    /**
     * @generated from field: string error = 2;
     */
    value: string;
    case: "error";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message script.Response.
 * Use `create(ResponseSchema)` to create a new message.
 */
export const ResponseSchema: GenMessage<Response> = /*@__PURE__*/
  messageDesc(file_script, 12);

/**
 * A response to a request
 *
 * @generated from message script.SendResponse
 */
export type SendResponse = Message$1<"script.SendResponse"> & {
  /**
   * Sequence number for request-response matching
   *
   * @generated from field: required uint64 seq = 1;
   */
  seq: bigint;

  /**
   * @generated from field: required script.Response response = 2;
   */
  response?: Response;
};

/**
 * Describes the message script.SendResponse.
 * Use `create(SendResponseSchema)` to create a new message.
 */
export const SendResponseSchema: GenMessage<SendResponse> = /*@__PURE__*/
  messageDesc(file_script, 13);

/**
 * A list of peers
 *
 * @generated from message script.Peers
 */
export type Peers = Message$1<"script.Peers"> & {
  /**
   * @generated from field: repeated script.Peer peers = 1;
   */
  peers: Peer[];
};

/**
 * Describes the message script.Peers.
 * Use `create(PeersSchema)` to create a new message.
 */
export const PeersSchema: GenMessage<Peers> = /*@__PURE__*/
  messageDesc(file_script, 14);

/**
 * A neighbour discovery event
 *
 * @generated from message script.NeighbourEvent
 */
export type NeighbourEvent = Message$1<"script.NeighbourEvent"> & {
  /**
   * @generated from oneof script.NeighbourEvent.event
   */
  event: {
    /**
     * @generated from field: script.Peers init = 1;
     */
    value: Peers;
    case: "init";
  } | {
    /**
     * @generated from field: script.Peer discovered = 2;
     */
    value: Peer;
    case: "discovered";
  } | {
    /**
     * @generated from field: script.Peer lost = 3;
     */
    value: Peer;
    case: "lost";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message script.NeighbourEvent.
 * Use `create(NeighbourEventSchema)` to create a new message.
 */
export const NeighbourEventSchema: GenMessage<NeighbourEvent> = /*@__PURE__*/
  messageDesc(file_script, 15);

/**
 * An ID of a message in gossipsub
 *
 * @generated from message script.GossipSubMessageID
 */
export type GossipSubMessageID = Message$1<"script.GossipSubMessageID"> & {
  /**
   * @generated from field: required bytes id = 1;
   */
  id: Uint8Array;
};

/**
 * Describes the message script.GossipSubMessageID.
 * Use `create(GossipSubMessageIDSchema)` to create a new message.
 */
export const GossipSubMessageIDSchema: GenMessage<GossipSubMessageID> = /*@__PURE__*/
  messageDesc(file_script, 16);

/**
 * A message for publishing in a gossipsub topic
 *
 * @generated from message script.GossipSubMessage
 */
export type GossipSubMessage = Message$1<"script.GossipSubMessage"> & {
  /**
   * @generated from field: required script.Data data = 1;
   */
  data?: Data;

  /**
   * @generated from field: required script.Topic topic = 2;
   */
  topic?: Topic;
};

/**
 * Describes the message script.GossipSubMessage.
 * Use `create(GossipSubMessageSchema)` to create a new message.
 */
export const GossipSubMessageSchema: GenMessage<GossipSubMessage> = /*@__PURE__*/
  messageDesc(file_script, 17);

/**
 * A received message from a gossipsub topic
 *
 * @generated from message script.GossipSubRecvMessage
 */
export type GossipSubRecvMessage = Message$1<"script.GossipSubRecvMessage"> & {
  /**
   * @generated from field: required script.Peer propagation_source = 1;
   */
  propagationSource?: Peer;

  /**
   * @generated from field: optional script.Peer source = 2;
   */
  source?: Peer;

  /**
   * @generated from field: required script.GossipSubMessage msg = 3;
   */
  msg?: GossipSubMessage;

  /**
   * @generated from field: required script.GossipSubMessageID msg_id = 4;
   */
  msgId?: GossipSubMessageID;
};

/**
 * Describes the message script.GossipSubRecvMessage.
 * Use `create(GossipSubRecvMessageSchema)` to create a new message.
 */
export const GossipSubRecvMessageSchema: GenMessage<GossipSubRecvMessage> = /*@__PURE__*/
  messageDesc(file_script, 18);

/**
 * A DHT key with a topic
 *
 * @generated from message script.DHTKey
 */
export type DHTKey = Message$1<"script.DHTKey"> & {
  /**
   * @generated from field: required script.Topic topic = 1;
   */
  topic?: Topic;

  /**
   * @generated from field: required bytes key = 2;
   */
  key: Uint8Array;
};

/**
 * Describes the message script.DHTKey.
 * Use `create(DHTKeySchema)` to create a new message.
 */
export const DHTKeySchema: GenMessage<DHTKey> = /*@__PURE__*/
  messageDesc(file_script, 19);

/**
 * A record for putting in the DHT
 *
 * @generated from message script.DHTRecord
 */
export type DHTRecord = Message$1<"script.DHTRecord"> & {
  /**
   * @generated from field: required script.DHTKey key = 1;
   */
  key?: DHTKey;

  /**
   * @generated from field: required script.Data value = 2;
   */
  value?: Data;
};

/**
 * Describes the message script.DHTRecord.
 * Use `create(DHTRecordSchema)` to create a new message.
 */
export const DHTRecordSchema: GenMessage<DHTRecord> = /*@__PURE__*/
  messageDesc(file_script, 20);

/**
 * A key-value pair for putting in the database
 *
 * @generated from message script.DBRecord
 */
export type DBRecord = Message$1<"script.DBRecord"> & {
  /**
   * @generated from field: required string key = 1;
   */
  key: string;

  /**
   * @generated from field: required script.Data value = 2;
   */
  value?: Data;
};

/**
 * Describes the message script.DBRecord.
 * Use `create(DBRecordSchema)` to create a new message.
 */
export const DBRecordSchema: GenMessage<DBRecord> = /*@__PURE__*/
  messageDesc(file_script, 21);

/**
 * A key for getting a value from the database
 *
 * @generated from message script.DBKey
 */
export type DBKey = Message$1<"script.DBKey"> & {
  /**
   * @generated from field: required string key = 1;
   */
  key: string;
};

/**
 * Describes the message script.DBKey.
 * Use `create(DBKeySchema)` to create a new message.
 */
export const DBKeySchema: GenMessage<DBKey> = /*@__PURE__*/
  messageDesc(file_script, 22);

/**
 * A file path on the local filesystem
 *
 * @generated from message script.FilePath
 */
export type FilePath = Message$1<"script.FilePath"> & {
  /**
   * @generated from field: required string path = 1;
   */
  path: string;
};

/**
 * Describes the message script.FilePath.
 * Use `create(FilePathSchema)` to create a new message.
 */
export const FilePathSchema: GenMessage<FilePath> = /*@__PURE__*/
  messageDesc(file_script, 23);

/**
 * The cid of a file
 *
 * @generated from message script.CID
 */
export type CID = Message$1<"script.CID"> & {
  /**
   * @generated from field: required bytes hash = 1;
   */
  hash: Uint8Array;

  /**
   * @generated from field: required script.ID id = 2;
   */
  id?: ID;
};

/**
 * Describes the message script.CID.
 * Use `create(CIDSchema)` to create a new message.
 */
export const CIDSchema: GenMessage<CID> = /*@__PURE__*/
  messageDesc(file_script, 24);

/**
 * A download event
 *
 * The progress field is a percentage of the download progress
 *
 * @generated from message script.DownloadEvent
 */
export type DownloadEvent = Message$1<"script.DownloadEvent"> & {
  /**
   * @generated from oneof script.DownloadEvent.event
   */
  event: {
    /**
     * @generated from field: uint64 progress = 1;
     */
    value: bigint;
    case: "progress";
  } | {
    /**
     * @generated from field: script.FilePath ready = 2;
     */
    value: FilePath;
    case: "ready";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message script.DownloadEvent.
 * Use `create(DownloadEventSchema)` to create a new message.
 */
export const DownloadEventSchema: GenMessage<DownloadEvent> = /*@__PURE__*/
  messageDesc(file_script, 25);

/**
 * A mesh topology event
 *
 * @generated from message script.MeshTopologyEvent
 */
export type MeshTopologyEvent = Message$1<"script.MeshTopologyEvent"> & {
  /**
   * @generated from field: required script.Peer peer = 1;
   */
  peer?: Peer;

  /**
   * @generated from field: required script.NeighbourEvent event = 2;
   */
  event?: NeighbourEvent;
};

/**
 * Describes the message script.MeshTopologyEvent.
 * Use `create(MeshTopologyEventSchema)` to create a new message.
 */
export const MeshTopologyEventSchema: GenMessage<MeshTopologyEvent> = /*@__PURE__*/
  messageDesc(file_script, 26);

/**
 * A request debug event
 *
 * @generated from message script.RequestDebugEvent
 */
export type RequestDebugEvent = Message$1<"script.RequestDebugEvent"> & {
  /**
   * @generated from field: required script.ID id = 1;
   */
  id?: ID;

  /**
   * @generated from field: required script.Peer receiver = 2;
   */
  receiver?: Peer;

  /**
   * @generated from field: required script.Message msg = 3;
   */
  msg?: Message;
};

/**
 * Describes the message script.RequestDebugEvent.
 * Use `create(RequestDebugEventSchema)` to create a new message.
 */
export const RequestDebugEventSchema: GenMessage<RequestDebugEvent> = /*@__PURE__*/
  messageDesc(file_script, 27);

/**
 * A response debug event
 *
 * @generated from message script.ResponseDebugEvent
 */
export type ResponseDebugEvent = Message$1<"script.ResponseDebugEvent"> & {
  /**
   * @generated from field: required script.ID req_id = 1;
   */
  reqId?: ID;

  /**
   * @generated from field: required script.Response response = 2;
   */
  response?: Response;
};

/**
 * Describes the message script.ResponseDebugEvent.
 * Use `create(ResponseDebugEventSchema)` to create a new message.
 */
export const ResponseDebugEventSchema: GenMessage<ResponseDebugEvent> = /*@__PURE__*/
  messageDesc(file_script, 28);

/**
 * A message debug event
 *
 * @generated from message script.MessageDebugEvent
 */
export type MessageDebugEvent = Message$1<"script.MessageDebugEvent"> & {
  /**
   * @generated from field: required script.Peer sender = 1;
   */
  sender?: Peer;

  /**
   * @generated from oneof script.MessageDebugEvent.event
   */
  event: {
    /**
     * @generated from field: script.RequestDebugEvent req = 2;
     */
    value: RequestDebugEvent;
    case: "req";
  } | {
    /**
     * @generated from field: script.ResponseDebugEvent res = 3;
     */
    value: ResponseDebugEvent;
    case: "res";
  } | {
    /**
     * @generated from field: script.GossipSubMessage gos = 4;
     */
    value: GossipSubMessage;
    case: "gos";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message script.MessageDebugEvent.
 * Use `create(MessageDebugEventSchema)` to create a new message.
 */
export const MessageDebugEventSchema: GenMessage<MessageDebugEvent> = /*@__PURE__*/
  messageDesc(file_script, 29);

/**
 * A docker image
 *
 * @generated from message script.DockerImage
 */
export type DockerImage = Message$1<"script.DockerImage"> & {
  /**
   * @generated from field: required string name = 1;
   */
  name: string;
};

/**
 * Describes the message script.DockerImage.
 * Use `create(DockerImageSchema)` to create a new message.
 */
export const DockerImageSchema: GenMessage<DockerImage> = /*@__PURE__*/
  messageDesc(file_script, 30);

/**
 * A docker script
 *
 * @generated from message script.DockerScript
 */
export type DockerScript = Message$1<"script.DockerScript"> & {
  /**
   * @generated from field: required script.DockerImage image = 1;
   */
  image?: DockerImage;

  /**
   * @generated from field: repeated uint32 ports = 2;
   */
  ports: number[];
};

/**
 * Describes the message script.DockerScript.
 * Use `create(DockerScriptSchema)` to create a new message.
 */
export const DockerScriptSchema: GenMessage<DockerScript> = /*@__PURE__*/
  messageDesc(file_script, 31);

/**
 * A request to deploy a script to a peer
 *
 * @generated from message script.DeployScriptRequest
 */
export type DeployScriptRequest = Message$1<"script.DeployScriptRequest"> & {
  /**
   * @generated from field: required script.DockerScript script = 1;
   */
  script?: DockerScript;

  /**
   * @generated from field: required bool local = 2;
   */
  local: boolean;

  /**
   * The peer can be empty if the script should be deployed to self
   *
   * @generated from field: optional script.Peer peer = 3;
   */
  peer?: Peer;

  /**
   * @generated from field: required bool persistent = 4;
   */
  persistent: boolean;
};

/**
 * Describes the message script.DeployScriptRequest.
 * Use `create(DeployScriptRequestSchema)` to create a new message.
 */
export const DeployScriptRequestSchema: GenMessage<DeployScriptRequest> = /*@__PURE__*/
  messageDesc(file_script, 32);

/**
 * A request to list running scripts on a peer
 *
 * @generated from message script.ListRunningScriptsRequest
 */
export type ListRunningScriptsRequest = Message$1<"script.ListRunningScriptsRequest"> & {
  /**
   * The peer can be empty if the scripts running on self should be listed
   *
   * @generated from field: optional script.Peer peer = 1;
   */
  peer?: Peer;
};

/**
 * Describes the message script.ListRunningScriptsRequest.
 * Use `create(ListRunningScriptsRequestSchema)` to create a new message.
 */
export const ListRunningScriptsRequestSchema: GenMessage<ListRunningScriptsRequest> = /*@__PURE__*/
  messageDesc(file_script, 33);

/**
 * A script that is running on a peer
 *
 * @generated from message script.RunningScript
 */
export type RunningScript = Message$1<"script.RunningScript"> & {
  /**
   * @generated from field: required script.ID id = 1;
   */
  id?: ID;

  /**
   * @generated from field: required script.DockerImage image = 2;
   */
  image?: DockerImage;

  /**
   * @generated from field: optional string name = 3;
   */
  name: string;
};

/**
 * Describes the message script.RunningScript.
 * Use `create(RunningScriptSchema)` to create a new message.
 */
export const RunningScriptSchema: GenMessage<RunningScript> = /*@__PURE__*/
  messageDesc(file_script, 34);

/**
 * A list of running scripts
 *
 * @generated from message script.RunningScripts
 */
export type RunningScripts = Message$1<"script.RunningScripts"> & {
  /**
   * @generated from field: repeated script.RunningScript scripts = 1;
   */
  scripts: RunningScript[];
};

/**
 * Describes the message script.RunningScripts.
 * Use `create(RunningScriptsSchema)` to create a new message.
 */
export const RunningScriptsSchema: GenMessage<RunningScripts> = /*@__PURE__*/
  messageDesc(file_script, 35);

/**
 * A request to stop a running script on a peer
 *
 * @generated from message script.StopScriptRequest
 */
export type StopScriptRequest = Message$1<"script.StopScriptRequest"> & {
  /**
   * @generated from field: required script.ID id = 1;
   */
  id?: ID;

  /**
   * The peer can be empty if the script should be stopped on self
   *
   * @generated from field: optional script.Peer peer = 2;
   */
  peer?: Peer;
};

/**
 * Describes the message script.StopScriptRequest.
 * Use `create(StopScriptRequestSchema)` to create a new message.
 */
export const StopScriptRequestSchema: GenMessage<StopScriptRequest> = /*@__PURE__*/
  messageDesc(file_script, 36);

/**
 * @generated from service script.ReqResp
 */
export const ReqResp: GenService<{
  /**
   * Send a request with an optional topic to a peer and wait for a response
   *
   * @generated from rpc script.ReqResp.Send
   */
  send: {
    methodKind: "unary";
    input: typeof SendRequestSchema;
    output: typeof ResponseSchema;
  },
  /**
   * Receive requests from peers that either have no topic or have a topic that
   * has been subscribed to
   *
   * @generated from rpc script.ReqResp.Recv
   */
  recv: {
    methodKind: "server_streaming";
    input: typeof OptionalTopicQuerySchema;
    output: typeof RecvRequestSchema;
  },
  /**
   * Respond to a request received from Recv
   *
   * @generated from rpc script.ReqResp.Respond
   */
  respond: {
    methodKind: "unary";
    input: typeof SendResponseSchema;
    output: typeof EmptySchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 0);

/**
 * @generated from service script.Discovery
 */
export const Discovery: GenService<{
  /**
   * Subscribe to neighbour discovery events to get notified when new neighbour
   * peers are discovered or lost
   *
   * @generated from rpc script.Discovery.SubscribeEvents
   */
  subscribeEvents: {
    methodKind: "server_streaming";
    input: typeof EmptySchema;
    output: typeof NeighbourEventSchema;
  },
  /**
   * Get the peer id of the current runtime
   *
   * @generated from rpc script.Discovery.GetOwnId
   */
  getOwnId: {
    methodKind: "unary";
    input: typeof EmptySchema;
    output: typeof PeerSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 1);

/**
 * @generated from service script.GossipSub
 */
export const GossipSub: GenService<{
  /**
   * Subscribe to a gossipsub topic to receive messages published in that topic
   *
   * @generated from rpc script.GossipSub.Subscribe
   */
  subscribe: {
    methodKind: "server_streaming";
    input: typeof TopicSchema;
    output: typeof GossipSubRecvMessageSchema;
  },
  /**
   * Publish a message in a gossipsub topic
   *
   * @generated from rpc script.GossipSub.Publish
   */
  publish: {
    methodKind: "unary";
    input: typeof GossipSubMessageSchema;
    output: typeof GossipSubMessageIDSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 2);

/**
 * @generated from service script.DHT
 */
export const DHT: GenService<{
  /**
   * Put a record in the DHT
   *
   * @generated from rpc script.DHT.PutRecord
   */
  putRecord: {
    methodKind: "unary";
    input: typeof DHTRecordSchema;
    output: typeof EmptySchema;
  },
  /**
   * Get a record from the DHT. The value of the record will be empty if the key
   * is not found.
   *
   * @generated from rpc script.DHT.GetRecord
   */
  getRecord: {
    methodKind: "unary";
    input: typeof DHTKeySchema;
    output: typeof OptionalDataSchema;
  },
  /**
   * Remove a record from the DHT
   * This only has local effects and only affects the DHT once the records expire.
   *
   * @generated from rpc script.DHT.RemoveRecord
   */
  removeRecord: {
    methodKind: "unary";
    input: typeof DHTKeySchema;
    output: typeof EmptySchema;
  },
  /**
   * Mark the current runtime as a provider for a key in the DHT
   *
   * @generated from rpc script.DHT.Provide
   */
  provide: {
    methodKind: "unary";
    input: typeof DHTKeySchema;
    output: typeof EmptySchema;
  },
  /**
   * Get the providers of a key in the DHT
   *
   * @generated from rpc script.DHT.GetProviders
   */
  getProviders: {
    methodKind: "server_streaming";
    input: typeof DHTKeySchema;
    output: typeof PeerSchema;
  },
  /**
   * Stop providing a key in the DHT.
   * This only has local effects and only affects the DHT once the providers records expire.
   *
   * @generated from rpc script.DHT.StopProviding
   */
  stopProviding: {
    methodKind: "unary";
    input: typeof DHTKeySchema;
    output: typeof EmptySchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 3);

/**
 * @generated from service script.DB
 */
export const DB: GenService<{
  /**
   * Put a record into the key-value store and get the previous value if it
   * exists
   *
   * @generated from rpc script.DB.Put
   */
  put: {
    methodKind: "unary";
    input: typeof DBRecordSchema;
    output: typeof OptionalDataSchema;
  },
  /**
   * Get a record from the key-value store
   *
   * @generated from rpc script.DB.Get
   */
  get: {
    methodKind: "unary";
    input: typeof DBKeySchema;
    output: typeof OptionalDataSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 4);

/**
 * @generated from service script.FileTransfer
 */
export const FileTransfer: GenService<{
  /**
   * Publish a file in the runtime and get the cid of the file
   *
   * @generated from rpc script.FileTransfer.PublishFile
   */
  publishFile: {
    methodKind: "unary";
    input: typeof FilePathSchema;
    output: typeof CIDSchema;
  },
  /**
   * Request a file with a cid from the runtime
   *
   * @generated from rpc script.FileTransfer.GetFile
   */
  getFile: {
    methodKind: "unary";
    input: typeof CIDSchema;
    output: typeof FilePathSchema;
  },
  /**
   * Request a file with a cid from the runtime and get notified about the
   * download progress
   *
   * @generated from rpc script.FileTransfer.GetFileWithProgress
   */
  getFileWithProgress: {
    methodKind: "server_streaming";
    input: typeof CIDSchema;
    output: typeof DownloadEventSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 5);

/**
 * @generated from service script.Debug
 */
export const Debug: GenService<{
  /**
   * Subscribe to mesh topology events to get notified when the mesh topology
   * changes
   *
   * @generated from rpc script.Debug.SubscribeMeshTopology
   */
  subscribeMeshTopology: {
    methodKind: "server_streaming";
    input: typeof EmptySchema;
    output: typeof MeshTopologyEventSchema;
  },
  /**
   * Subscribe to message debug events to get notified when messages are sent
   *
   * @generated from rpc script.Debug.SubscribeMessages
   */
  subscribeMessages: {
    methodKind: "server_streaming";
    input: typeof EmptySchema;
    output: typeof MessageDebugEventSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 6);

/**
 * @generated from service script.Scripting
 */
export const Scripting: GenService<{
  /**
   * Deploy a script to a peer and get the id of the deployed script
   *
   * @generated from rpc script.Scripting.DeployScript
   */
  deployScript: {
    methodKind: "unary";
    input: typeof DeployScriptRequestSchema;
    output: typeof IDSchema;
  },
  /**
   * List running scripts on a peer
   *
   * @generated from rpc script.Scripting.ListRunningScripts
   */
  listRunningScripts: {
    methodKind: "unary";
    input: typeof ListRunningScriptsRequestSchema;
    output: typeof RunningScriptsSchema;
  },
  /**
   * Stop a running script on a peer
   *
   * @generated from rpc script.Scripting.StopScript
   */
  stopScript: {
    methodKind: "unary";
    input: typeof StopScriptRequestSchema;
    output: typeof EmptySchema;
  },
  /**
   * Get the id of the current script
   *
   * @generated from rpc script.Scripting.GetOwnId
   */
  getOwnId: {
    methodKind: "unary";
    input: typeof EmptySchema;
    output: typeof IDSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_script, 7);

