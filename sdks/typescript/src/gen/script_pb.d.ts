import * as jspb from 'google-protobuf'



export class Empty extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Empty.AsObject;
  static toObject(includeInstance: boolean, msg: Empty): Empty.AsObject;
  static serializeBinaryToWriter(message: Empty, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Empty;
  static deserializeBinaryFromReader(message: Empty, reader: jspb.BinaryReader): Empty;
}

export namespace Empty {
  export type AsObject = {
  }
}

export class Data extends jspb.Message {
  getData(): Uint8Array | string;
  getData_asU8(): Uint8Array;
  getData_asB64(): string;
  setData(value: Uint8Array | string): Data;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Data.AsObject;
  static toObject(includeInstance: boolean, msg: Data): Data.AsObject;
  static serializeBinaryToWriter(message: Data, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Data;
  static deserializeBinaryFromReader(message: Data, reader: jspb.BinaryReader): Data;
}

export namespace Data {
  export type AsObject = {
    data: Uint8Array | string,
  }
}

export class OptionalData extends jspb.Message {
  getData(): Data | undefined;
  setData(value?: Data): OptionalData;
  hasData(): boolean;
  clearData(): OptionalData;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OptionalData.AsObject;
  static toObject(includeInstance: boolean, msg: OptionalData): OptionalData.AsObject;
  static serializeBinaryToWriter(message: OptionalData, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OptionalData;
  static deserializeBinaryFromReader(message: OptionalData, reader: jspb.BinaryReader): OptionalData;
}

export namespace OptionalData {
  export type AsObject = {
    data?: Data.AsObject,
  }
}

export class ID extends jspb.Message {
  getUlid(): string;
  setUlid(value: string): ID;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ID.AsObject;
  static toObject(includeInstance: boolean, msg: ID): ID.AsObject;
  static serializeBinaryToWriter(message: ID, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ID;
  static deserializeBinaryFromReader(message: ID, reader: jspb.BinaryReader): ID;
}

export namespace ID {
  export type AsObject = {
    ulid: string,
  }
}

export class Peer extends jspb.Message {
  getPeerId(): string;
  setPeerId(value: string): Peer;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Peer.AsObject;
  static toObject(includeInstance: boolean, msg: Peer): Peer.AsObject;
  static serializeBinaryToWriter(message: Peer, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Peer;
  static deserializeBinaryFromReader(message: Peer, reader: jspb.BinaryReader): Peer;
}

export namespace Peer {
  export type AsObject = {
    peerId: string,
  }
}

export class Topic extends jspb.Message {
  getTopic(): string;
  setTopic(value: string): Topic;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Topic.AsObject;
  static toObject(includeInstance: boolean, msg: Topic): Topic.AsObject;
  static serializeBinaryToWriter(message: Topic, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Topic;
  static deserializeBinaryFromReader(message: Topic, reader: jspb.BinaryReader): Topic;
}

export namespace Topic {
  export type AsObject = {
    topic: string,
  }
}

export class OptionalTopic extends jspb.Message {
  getTopic(): Topic | undefined;
  setTopic(value?: Topic): OptionalTopic;
  hasTopic(): boolean;
  clearTopic(): OptionalTopic;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OptionalTopic.AsObject;
  static toObject(includeInstance: boolean, msg: OptionalTopic): OptionalTopic.AsObject;
  static serializeBinaryToWriter(message: OptionalTopic, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OptionalTopic;
  static deserializeBinaryFromReader(message: OptionalTopic, reader: jspb.BinaryReader): OptionalTopic;
}

export namespace OptionalTopic {
  export type AsObject = {
    topic?: Topic.AsObject,
  }
}

export class TopicQuery extends jspb.Message {
  getTopic(): Topic | undefined;
  setTopic(value?: Topic): TopicQuery;
  hasTopic(): boolean;
  clearTopic(): TopicQuery;

  getRegex(): string;
  setRegex(value: string): TopicQuery;

  getQueryCase(): TopicQuery.QueryCase;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TopicQuery.AsObject;
  static toObject(includeInstance: boolean, msg: TopicQuery): TopicQuery.AsObject;
  static serializeBinaryToWriter(message: TopicQuery, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TopicQuery;
  static deserializeBinaryFromReader(message: TopicQuery, reader: jspb.BinaryReader): TopicQuery;
}

export namespace TopicQuery {
  export type AsObject = {
    topic?: Topic.AsObject,
    regex: string,
  }

  export enum QueryCase { 
    QUERY_NOT_SET = 0,
    TOPIC = 1,
    REGEX = 2,
  }
}

export class OptionalTopicQuery extends jspb.Message {
  getQuery(): TopicQuery | undefined;
  setQuery(value?: TopicQuery): OptionalTopicQuery;
  hasQuery(): boolean;
  clearQuery(): OptionalTopicQuery;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OptionalTopicQuery.AsObject;
  static toObject(includeInstance: boolean, msg: OptionalTopicQuery): OptionalTopicQuery.AsObject;
  static serializeBinaryToWriter(message: OptionalTopicQuery, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OptionalTopicQuery;
  static deserializeBinaryFromReader(message: OptionalTopicQuery, reader: jspb.BinaryReader): OptionalTopicQuery;
}

export namespace OptionalTopicQuery {
  export type AsObject = {
    query?: TopicQuery.AsObject,
  }
}

export class Message extends jspb.Message {
  getData(): Data | undefined;
  setData(value?: Data): Message;
  hasData(): boolean;
  clearData(): Message;

  getTopic(): OptionalTopic | undefined;
  setTopic(value?: OptionalTopic): Message;
  hasTopic(): boolean;
  clearTopic(): Message;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Message.AsObject;
  static toObject(includeInstance: boolean, msg: Message): Message.AsObject;
  static serializeBinaryToWriter(message: Message, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Message;
  static deserializeBinaryFromReader(message: Message, reader: jspb.BinaryReader): Message;
}

export namespace Message {
  export type AsObject = {
    data?: Data.AsObject,
    topic?: OptionalTopic.AsObject,
  }
}

export class SendRequest extends jspb.Message {
  getPeer(): Peer | undefined;
  setPeer(value?: Peer): SendRequest;
  hasPeer(): boolean;
  clearPeer(): SendRequest;

  getMsg(): Message | undefined;
  setMsg(value?: Message): SendRequest;
  hasMsg(): boolean;
  clearMsg(): SendRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SendRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SendRequest): SendRequest.AsObject;
  static serializeBinaryToWriter(message: SendRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SendRequest;
  static deserializeBinaryFromReader(message: SendRequest, reader: jspb.BinaryReader): SendRequest;
}

export namespace SendRequest {
  export type AsObject = {
    peer?: Peer.AsObject,
    msg?: Message.AsObject,
  }
}

export class RecvRequest extends jspb.Message {
  getPeer(): Peer | undefined;
  setPeer(value?: Peer): RecvRequest;
  hasPeer(): boolean;
  clearPeer(): RecvRequest;

  getMsg(): Message | undefined;
  setMsg(value?: Message): RecvRequest;
  hasMsg(): boolean;
  clearMsg(): RecvRequest;

  getSeq(): number;
  setSeq(value: number): RecvRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RecvRequest.AsObject;
  static toObject(includeInstance: boolean, msg: RecvRequest): RecvRequest.AsObject;
  static serializeBinaryToWriter(message: RecvRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RecvRequest;
  static deserializeBinaryFromReader(message: RecvRequest, reader: jspb.BinaryReader): RecvRequest;
}

export namespace RecvRequest {
  export type AsObject = {
    peer?: Peer.AsObject,
    msg?: Message.AsObject,
    seq: number,
  }
}

export class Response extends jspb.Message {
  getData(): Data | undefined;
  setData(value?: Data): Response;
  hasData(): boolean;
  clearData(): Response;

  getError(): string;
  setError(value: string): Response;

  getResponseCase(): Response.ResponseCase;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Response.AsObject;
  static toObject(includeInstance: boolean, msg: Response): Response.AsObject;
  static serializeBinaryToWriter(message: Response, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Response;
  static deserializeBinaryFromReader(message: Response, reader: jspb.BinaryReader): Response;
}

export namespace Response {
  export type AsObject = {
    data?: Data.AsObject,
    error: string,
  }

  export enum ResponseCase { 
    RESPONSE_NOT_SET = 0,
    DATA = 1,
    ERROR = 2,
  }
}

export class SendResponse extends jspb.Message {
  getSeq(): number;
  setSeq(value: number): SendResponse;

  getResponse(): Response | undefined;
  setResponse(value?: Response): SendResponse;
  hasResponse(): boolean;
  clearResponse(): SendResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SendResponse.AsObject;
  static toObject(includeInstance: boolean, msg: SendResponse): SendResponse.AsObject;
  static serializeBinaryToWriter(message: SendResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SendResponse;
  static deserializeBinaryFromReader(message: SendResponse, reader: jspb.BinaryReader): SendResponse;
}

export namespace SendResponse {
  export type AsObject = {
    seq: number,
    response?: Response.AsObject,
  }
}

export class Peers extends jspb.Message {
  getPeersList(): Array<Peer>;
  setPeersList(value: Array<Peer>): Peers;
  clearPeersList(): Peers;
  addPeers(value?: Peer, index?: number): Peer;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Peers.AsObject;
  static toObject(includeInstance: boolean, msg: Peers): Peers.AsObject;
  static serializeBinaryToWriter(message: Peers, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Peers;
  static deserializeBinaryFromReader(message: Peers, reader: jspb.BinaryReader): Peers;
}

export namespace Peers {
  export type AsObject = {
    peersList: Array<Peer.AsObject>,
  }
}

export class NeighbourEvent extends jspb.Message {
  getInit(): Peers | undefined;
  setInit(value?: Peers): NeighbourEvent;
  hasInit(): boolean;
  clearInit(): NeighbourEvent;

  getDiscovered(): Peer | undefined;
  setDiscovered(value?: Peer): NeighbourEvent;
  hasDiscovered(): boolean;
  clearDiscovered(): NeighbourEvent;

  getLost(): Peer | undefined;
  setLost(value?: Peer): NeighbourEvent;
  hasLost(): boolean;
  clearLost(): NeighbourEvent;

  getEventCase(): NeighbourEvent.EventCase;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): NeighbourEvent.AsObject;
  static toObject(includeInstance: boolean, msg: NeighbourEvent): NeighbourEvent.AsObject;
  static serializeBinaryToWriter(message: NeighbourEvent, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): NeighbourEvent;
  static deserializeBinaryFromReader(message: NeighbourEvent, reader: jspb.BinaryReader): NeighbourEvent;
}

export namespace NeighbourEvent {
  export type AsObject = {
    init?: Peers.AsObject,
    discovered?: Peer.AsObject,
    lost?: Peer.AsObject,
  }

  export enum EventCase { 
    EVENT_NOT_SET = 0,
    INIT = 1,
    DISCOVERED = 2,
    LOST = 3,
  }
}

export class GossipSubMessageID extends jspb.Message {
  getId(): Uint8Array | string;
  getId_asU8(): Uint8Array;
  getId_asB64(): string;
  setId(value: Uint8Array | string): GossipSubMessageID;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GossipSubMessageID.AsObject;
  static toObject(includeInstance: boolean, msg: GossipSubMessageID): GossipSubMessageID.AsObject;
  static serializeBinaryToWriter(message: GossipSubMessageID, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GossipSubMessageID;
  static deserializeBinaryFromReader(message: GossipSubMessageID, reader: jspb.BinaryReader): GossipSubMessageID;
}

export namespace GossipSubMessageID {
  export type AsObject = {
    id: Uint8Array | string,
  }
}

export class GossipSubMessage extends jspb.Message {
  getData(): Data | undefined;
  setData(value?: Data): GossipSubMessage;
  hasData(): boolean;
  clearData(): GossipSubMessage;

  getTopic(): Topic | undefined;
  setTopic(value?: Topic): GossipSubMessage;
  hasTopic(): boolean;
  clearTopic(): GossipSubMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GossipSubMessage.AsObject;
  static toObject(includeInstance: boolean, msg: GossipSubMessage): GossipSubMessage.AsObject;
  static serializeBinaryToWriter(message: GossipSubMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GossipSubMessage;
  static deserializeBinaryFromReader(message: GossipSubMessage, reader: jspb.BinaryReader): GossipSubMessage;
}

export namespace GossipSubMessage {
  export type AsObject = {
    data?: Data.AsObject,
    topic?: Topic.AsObject,
  }
}

export class GossipSubRecvMessage extends jspb.Message {
  getPropagationSource(): Peer | undefined;
  setPropagationSource(value?: Peer): GossipSubRecvMessage;
  hasPropagationSource(): boolean;
  clearPropagationSource(): GossipSubRecvMessage;

  getSource(): Peer | undefined;
  setSource(value?: Peer): GossipSubRecvMessage;
  hasSource(): boolean;
  clearSource(): GossipSubRecvMessage;

  getMsg(): GossipSubMessage | undefined;
  setMsg(value?: GossipSubMessage): GossipSubRecvMessage;
  hasMsg(): boolean;
  clearMsg(): GossipSubRecvMessage;

  getMsgId(): GossipSubMessageID | undefined;
  setMsgId(value?: GossipSubMessageID): GossipSubRecvMessage;
  hasMsgId(): boolean;
  clearMsgId(): GossipSubRecvMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GossipSubRecvMessage.AsObject;
  static toObject(includeInstance: boolean, msg: GossipSubRecvMessage): GossipSubRecvMessage.AsObject;
  static serializeBinaryToWriter(message: GossipSubRecvMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GossipSubRecvMessage;
  static deserializeBinaryFromReader(message: GossipSubRecvMessage, reader: jspb.BinaryReader): GossipSubRecvMessage;
}

export namespace GossipSubRecvMessage {
  export type AsObject = {
    propagationSource?: Peer.AsObject,
    source?: Peer.AsObject,
    msg?: GossipSubMessage.AsObject,
    msgId?: GossipSubMessageID.AsObject,
  }
}

export class DHTKey extends jspb.Message {
  getTopic(): Topic | undefined;
  setTopic(value?: Topic): DHTKey;
  hasTopic(): boolean;
  clearTopic(): DHTKey;

  getKey(): Uint8Array | string;
  getKey_asU8(): Uint8Array;
  getKey_asB64(): string;
  setKey(value: Uint8Array | string): DHTKey;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DHTKey.AsObject;
  static toObject(includeInstance: boolean, msg: DHTKey): DHTKey.AsObject;
  static serializeBinaryToWriter(message: DHTKey, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DHTKey;
  static deserializeBinaryFromReader(message: DHTKey, reader: jspb.BinaryReader): DHTKey;
}

export namespace DHTKey {
  export type AsObject = {
    topic?: Topic.AsObject,
    key: Uint8Array | string,
  }
}

export class DHTRecord extends jspb.Message {
  getKey(): DHTKey | undefined;
  setKey(value?: DHTKey): DHTRecord;
  hasKey(): boolean;
  clearKey(): DHTRecord;

  getValue(): Data | undefined;
  setValue(value?: Data): DHTRecord;
  hasValue(): boolean;
  clearValue(): DHTRecord;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DHTRecord.AsObject;
  static toObject(includeInstance: boolean, msg: DHTRecord): DHTRecord.AsObject;
  static serializeBinaryToWriter(message: DHTRecord, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DHTRecord;
  static deserializeBinaryFromReader(message: DHTRecord, reader: jspb.BinaryReader): DHTRecord;
}

export namespace DHTRecord {
  export type AsObject = {
    key?: DHTKey.AsObject,
    value?: Data.AsObject,
  }
}

export class DBRecord extends jspb.Message {
  getKey(): string;
  setKey(value: string): DBRecord;

  getValue(): Data | undefined;
  setValue(value?: Data): DBRecord;
  hasValue(): boolean;
  clearValue(): DBRecord;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DBRecord.AsObject;
  static toObject(includeInstance: boolean, msg: DBRecord): DBRecord.AsObject;
  static serializeBinaryToWriter(message: DBRecord, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DBRecord;
  static deserializeBinaryFromReader(message: DBRecord, reader: jspb.BinaryReader): DBRecord;
}

export namespace DBRecord {
  export type AsObject = {
    key: string,
    value?: Data.AsObject,
  }
}

export class DBKey extends jspb.Message {
  getKey(): string;
  setKey(value: string): DBKey;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DBKey.AsObject;
  static toObject(includeInstance: boolean, msg: DBKey): DBKey.AsObject;
  static serializeBinaryToWriter(message: DBKey, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DBKey;
  static deserializeBinaryFromReader(message: DBKey, reader: jspb.BinaryReader): DBKey;
}

export namespace DBKey {
  export type AsObject = {
    key: string,
  }
}

export class FilePath extends jspb.Message {
  getPath(): string;
  setPath(value: string): FilePath;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FilePath.AsObject;
  static toObject(includeInstance: boolean, msg: FilePath): FilePath.AsObject;
  static serializeBinaryToWriter(message: FilePath, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FilePath;
  static deserializeBinaryFromReader(message: FilePath, reader: jspb.BinaryReader): FilePath;
}

export namespace FilePath {
  export type AsObject = {
    path: string,
  }
}

export class CID extends jspb.Message {
  getHash(): Uint8Array | string;
  getHash_asU8(): Uint8Array;
  getHash_asB64(): string;
  setHash(value: Uint8Array | string): CID;

  getId(): ID | undefined;
  setId(value?: ID): CID;
  hasId(): boolean;
  clearId(): CID;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CID.AsObject;
  static toObject(includeInstance: boolean, msg: CID): CID.AsObject;
  static serializeBinaryToWriter(message: CID, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CID;
  static deserializeBinaryFromReader(message: CID, reader: jspb.BinaryReader): CID;
}

export namespace CID {
  export type AsObject = {
    hash: Uint8Array | string,
    id?: ID.AsObject,
  }
}

export class MeshTopologyEvent extends jspb.Message {
  getPeer(): Peer | undefined;
  setPeer(value?: Peer): MeshTopologyEvent;
  hasPeer(): boolean;
  clearPeer(): MeshTopologyEvent;

  getEvent(): NeighbourEvent | undefined;
  setEvent(value?: NeighbourEvent): MeshTopologyEvent;
  hasEvent(): boolean;
  clearEvent(): MeshTopologyEvent;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MeshTopologyEvent.AsObject;
  static toObject(includeInstance: boolean, msg: MeshTopologyEvent): MeshTopologyEvent.AsObject;
  static serializeBinaryToWriter(message: MeshTopologyEvent, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MeshTopologyEvent;
  static deserializeBinaryFromReader(message: MeshTopologyEvent, reader: jspb.BinaryReader): MeshTopologyEvent;
}

export namespace MeshTopologyEvent {
  export type AsObject = {
    peer?: Peer.AsObject,
    event?: NeighbourEvent.AsObject,
  }
}

export class RequestDebugEvent extends jspb.Message {
  getId(): ID | undefined;
  setId(value?: ID): RequestDebugEvent;
  hasId(): boolean;
  clearId(): RequestDebugEvent;

  getReceiver(): Peer | undefined;
  setReceiver(value?: Peer): RequestDebugEvent;
  hasReceiver(): boolean;
  clearReceiver(): RequestDebugEvent;

  getMsg(): Message | undefined;
  setMsg(value?: Message): RequestDebugEvent;
  hasMsg(): boolean;
  clearMsg(): RequestDebugEvent;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RequestDebugEvent.AsObject;
  static toObject(includeInstance: boolean, msg: RequestDebugEvent): RequestDebugEvent.AsObject;
  static serializeBinaryToWriter(message: RequestDebugEvent, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RequestDebugEvent;
  static deserializeBinaryFromReader(message: RequestDebugEvent, reader: jspb.BinaryReader): RequestDebugEvent;
}

export namespace RequestDebugEvent {
  export type AsObject = {
    id?: ID.AsObject,
    receiver?: Peer.AsObject,
    msg?: Message.AsObject,
  }
}

export class ResponseDebugEvent extends jspb.Message {
  getReqId(): ID | undefined;
  setReqId(value?: ID): ResponseDebugEvent;
  hasReqId(): boolean;
  clearReqId(): ResponseDebugEvent;

  getResponse(): Response | undefined;
  setResponse(value?: Response): ResponseDebugEvent;
  hasResponse(): boolean;
  clearResponse(): ResponseDebugEvent;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ResponseDebugEvent.AsObject;
  static toObject(includeInstance: boolean, msg: ResponseDebugEvent): ResponseDebugEvent.AsObject;
  static serializeBinaryToWriter(message: ResponseDebugEvent, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ResponseDebugEvent;
  static deserializeBinaryFromReader(message: ResponseDebugEvent, reader: jspb.BinaryReader): ResponseDebugEvent;
}

export namespace ResponseDebugEvent {
  export type AsObject = {
    reqId?: ID.AsObject,
    response?: Response.AsObject,
  }
}

export class MessageDebugEvent extends jspb.Message {
  getSender(): Peer | undefined;
  setSender(value?: Peer): MessageDebugEvent;
  hasSender(): boolean;
  clearSender(): MessageDebugEvent;

  getReq(): RequestDebugEvent | undefined;
  setReq(value?: RequestDebugEvent): MessageDebugEvent;
  hasReq(): boolean;
  clearReq(): MessageDebugEvent;

  getRes(): ResponseDebugEvent | undefined;
  setRes(value?: ResponseDebugEvent): MessageDebugEvent;
  hasRes(): boolean;
  clearRes(): MessageDebugEvent;

  getGos(): GossipSubMessage | undefined;
  setGos(value?: GossipSubMessage): MessageDebugEvent;
  hasGos(): boolean;
  clearGos(): MessageDebugEvent;

  getEventCase(): MessageDebugEvent.EventCase;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MessageDebugEvent.AsObject;
  static toObject(includeInstance: boolean, msg: MessageDebugEvent): MessageDebugEvent.AsObject;
  static serializeBinaryToWriter(message: MessageDebugEvent, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MessageDebugEvent;
  static deserializeBinaryFromReader(message: MessageDebugEvent, reader: jspb.BinaryReader): MessageDebugEvent;
}

export namespace MessageDebugEvent {
  export type AsObject = {
    sender?: Peer.AsObject,
    req?: RequestDebugEvent.AsObject,
    res?: ResponseDebugEvent.AsObject,
    gos?: GossipSubMessage.AsObject,
  }

  export enum EventCase { 
    EVENT_NOT_SET = 0,
    REQ = 2,
    RES = 3,
    GOS = 4,
  }
}

export class DockerImage extends jspb.Message {
  getName(): string;
  setName(value: string): DockerImage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DockerImage.AsObject;
  static toObject(includeInstance: boolean, msg: DockerImage): DockerImage.AsObject;
  static serializeBinaryToWriter(message: DockerImage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DockerImage;
  static deserializeBinaryFromReader(message: DockerImage, reader: jspb.BinaryReader): DockerImage;
}

export namespace DockerImage {
  export type AsObject = {
    name: string,
  }
}

export class DockerScript extends jspb.Message {
  getImage(): DockerImage | undefined;
  setImage(value?: DockerImage): DockerScript;
  hasImage(): boolean;
  clearImage(): DockerScript;

  getPortsList(): Array<number>;
  setPortsList(value: Array<number>): DockerScript;
  clearPortsList(): DockerScript;
  addPorts(value: number, index?: number): DockerScript;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DockerScript.AsObject;
  static toObject(includeInstance: boolean, msg: DockerScript): DockerScript.AsObject;
  static serializeBinaryToWriter(message: DockerScript, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DockerScript;
  static deserializeBinaryFromReader(message: DockerScript, reader: jspb.BinaryReader): DockerScript;
}

export namespace DockerScript {
  export type AsObject = {
    image?: DockerImage.AsObject,
    portsList: Array<number>,
  }
}

export class DeployScriptRequest extends jspb.Message {
  getScript(): DockerScript | undefined;
  setScript(value?: DockerScript): DeployScriptRequest;
  hasScript(): boolean;
  clearScript(): DeployScriptRequest;

  getLocal(): boolean;
  setLocal(value: boolean): DeployScriptRequest;

  getPeer(): Peer | undefined;
  setPeer(value?: Peer): DeployScriptRequest;
  hasPeer(): boolean;
  clearPeer(): DeployScriptRequest;

  getPersistent(): boolean;
  setPersistent(value: boolean): DeployScriptRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeployScriptRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeployScriptRequest): DeployScriptRequest.AsObject;
  static serializeBinaryToWriter(message: DeployScriptRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeployScriptRequest;
  static deserializeBinaryFromReader(message: DeployScriptRequest, reader: jspb.BinaryReader): DeployScriptRequest;
}

export namespace DeployScriptRequest {
  export type AsObject = {
    script?: DockerScript.AsObject,
    local: boolean,
    peer?: Peer.AsObject,
    persistent: boolean,
  }
}

export class ListRunningScriptsRequest extends jspb.Message {
  getPeer(): Peer | undefined;
  setPeer(value?: Peer): ListRunningScriptsRequest;
  hasPeer(): boolean;
  clearPeer(): ListRunningScriptsRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListRunningScriptsRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListRunningScriptsRequest): ListRunningScriptsRequest.AsObject;
  static serializeBinaryToWriter(message: ListRunningScriptsRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListRunningScriptsRequest;
  static deserializeBinaryFromReader(message: ListRunningScriptsRequest, reader: jspb.BinaryReader): ListRunningScriptsRequest;
}

export namespace ListRunningScriptsRequest {
  export type AsObject = {
    peer?: Peer.AsObject,
  }
}

export class RunningScript extends jspb.Message {
  getId(): ID | undefined;
  setId(value?: ID): RunningScript;
  hasId(): boolean;
  clearId(): RunningScript;

  getImage(): DockerImage | undefined;
  setImage(value?: DockerImage): RunningScript;
  hasImage(): boolean;
  clearImage(): RunningScript;

  getName(): string;
  setName(value: string): RunningScript;
  hasName(): boolean;
  clearName(): RunningScript;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RunningScript.AsObject;
  static toObject(includeInstance: boolean, msg: RunningScript): RunningScript.AsObject;
  static serializeBinaryToWriter(message: RunningScript, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RunningScript;
  static deserializeBinaryFromReader(message: RunningScript, reader: jspb.BinaryReader): RunningScript;
}

export namespace RunningScript {
  export type AsObject = {
    id?: ID.AsObject,
    image?: DockerImage.AsObject,
    name?: string,
  }
}

export class RunningScripts extends jspb.Message {
  getScriptsList(): Array<RunningScript>;
  setScriptsList(value: Array<RunningScript>): RunningScripts;
  clearScriptsList(): RunningScripts;
  addScripts(value?: RunningScript, index?: number): RunningScript;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RunningScripts.AsObject;
  static toObject(includeInstance: boolean, msg: RunningScripts): RunningScripts.AsObject;
  static serializeBinaryToWriter(message: RunningScripts, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RunningScripts;
  static deserializeBinaryFromReader(message: RunningScripts, reader: jspb.BinaryReader): RunningScripts;
}

export namespace RunningScripts {
  export type AsObject = {
    scriptsList: Array<RunningScript.AsObject>,
  }
}

export class StopScriptRequest extends jspb.Message {
  getId(): ID | undefined;
  setId(value?: ID): StopScriptRequest;
  hasId(): boolean;
  clearId(): StopScriptRequest;

  getPeer(): Peer | undefined;
  setPeer(value?: Peer): StopScriptRequest;
  hasPeer(): boolean;
  clearPeer(): StopScriptRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): StopScriptRequest.AsObject;
  static toObject(includeInstance: boolean, msg: StopScriptRequest): StopScriptRequest.AsObject;
  static serializeBinaryToWriter(message: StopScriptRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): StopScriptRequest;
  static deserializeBinaryFromReader(message: StopScriptRequest, reader: jspb.BinaryReader): StopScriptRequest;
}

export namespace StopScriptRequest {
  export type AsObject = {
    id?: ID.AsObject,
    peer?: Peer.AsObject,
  }
}

