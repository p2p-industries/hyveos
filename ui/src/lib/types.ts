export enum DeviceStatusTag {
  Ok,
  Error
}

export type DeviceStatusOk = {
  tag: DeviceStatusTag.Ok;
};

export type DeviceStatusError = {
  tag: DeviceStatusTag.Error;
  error: string;
};

export type DeviceStatus = DeviceStatusOk | DeviceStatusError;

export type DeviceType = {
  id: string;
  name: string;
  status: DeviceStatus;
  dockerImages: { name: string; status: 'running' | 'stopped' }[];
};

export type Node = {
  peerId: string;
  label: string;
};

export type Link = {
  source: string;
  target: string;
};

export type Graph = {
  nodes: Node[];
  links: Link[];
};

export interface EventData {
  peerId: string;
  events: Array<{ eventType: string; peerId: string[] }>;
}

export interface TopologyUpdateData {
  events: EventData[];
}

export type RequestResponse = {
  type: 'req_resp';
  id: string;
  topic: string;
  data: Uint8Array;
  receiver: string;
  response?:
    | {
        success: true;
        data: Uint8Array;
      }
    | {
        success: false;
        error: string;
      };
};

export type GossipsubMessage = {
  type: 'gossipsub';
  topic: string;
  message: string;
};

export type Message = {
  sender: string;
  data: RequestResponse | GossipsubMessage;
};
