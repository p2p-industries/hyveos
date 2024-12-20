import type { Transport } from 'npm:@connectrpc/connect'
import { AbortOnDispose, BaseService } from './core.ts'
import type { Event as NeighbourEvent } from './discovery.ts'
import {
  Debug as Service,
  type MeshTopologyEvent as MeshTopEvent,
  type MessageDebugEvent as MessageDbgEvent,
} from './gen/script_pb.ts'

export interface MeshTopologyEvent {
  peer: string
  neighbour: NeighbourEvent
}

export class MeshTopologySubscription extends AbortOnDispose
  implements AsyncIterable<MeshTopologyEvent>, Disposable {
  stream: AsyncIterable<MeshTopEvent>

  constructor(
    stream: AsyncIterable<MeshTopEvent>,
    abortController: AbortController,
  ) {
    super(abortController)
    this.stream = stream
  }

  async *[Symbol.asyncIterator](): AsyncIterator<MeshTopologyEvent> {
    for await (const { event, peer } of this.stream) {
      if (!peer) continue
      if (!event) continue
      const { peerId } = peer
      const { event: { case: evCase, value } } = event
      switch (evCase) {
        case 'init': {
          const peers = value.peers.map((peer) => peer.peerId)
          yield { peer: peerId, neighbour: { case: 'init', peers } }
          break
        }
        case 'discovered':
          yield {
            peer: peerId,
            neighbour: { case: 'discovered', peer: value.peerId },
          }
          break
        case 'lost':
          yield {
            peer: peerId,
            neighbour: { case: 'lost', peer: value.peerId },
          }
          break
      }
    }
  }
}

export type InnerDebugEvent = {
  case: 'req'
  id: string
  topic?: string
  data: Uint8Array
} | {
  case: 'gos'
  data: Uint8Array
  topic: string
} | {
  case: 'res'
  response: {
    success: boolean
    data: Uint8Array
  } | {
    success: false
    error: string
  }
}

export interface MessageDebugEvent {
  sender: string
  event: InnerDebugEvent
}

export class MessageSubscription extends AbortOnDispose {
  stream: AsyncIterable<MessageDbgEvent>

  constructor(
    stream: AsyncIterable<MessageDbgEvent>,
    abortController: AbortController,
  ) {
    super(abortController)
    this.stream = stream
  }

  async *[Symbol.asyncIterator](): AsyncIterator<MessageDebugEvent> {
    for await (const { event, sender } of this.stream) {
      const senderPeer = sender?.peerId
      if (!senderPeer) continue
      if (!event) continue
      const { case: evCase, value } = event
      switch (evCase) {
        case 'req': {
          if (!value.id) continue
          const { ulid } = value.id
          const topic = value.msg?.topic?.topic?.topic
          const data = value.msg?.data?.data
          if (!data) continue
          yield {
            sender: senderPeer,
            event: { case: 'req', id: ulid, topic, data },
          }
          break
        }
        case 'gos': {
          const data = value.data?.data
          if (!data) continue
          const topic = value.topic?.topic
          if (!topic) continue
          yield {
            sender: senderPeer,
            event: { case: 'gos', data, topic },
          }
          break
        }
        case 'res': {
          const ulid = value.reqId?.ulid
          if (!ulid) continue
          const response = value.response?.response
          if (!response) continue
          switch (response.case) {
            case 'data': {
              yield {
                sender: senderPeer,
                event: {
                  case: 'res',
                  response: { success: true, data: response.value.data },
                },
              }
              break
            }
            case 'error': {
              const error = response.value
              yield {
                sender: senderPeer,
                event: {
                  case: 'res',
                  response: { success: false, error },
                },
              }
              break
            }
          }
          break
        }
      }
    }
  }
}

export class Debug extends BaseService<typeof Service> {
  public static __create(transport: Transport): Debug {
    return new Debug(Service, transport)
  }

  public subscribeMeshTopology(): MeshTopologySubscription {
    const abortController = new AbortController()
    const stream = this.client.subscribeMeshTopology({}, {
      signal: abortController.signal,
    })
    return new MeshTopologySubscription(stream, abortController)
  }

  public subscribeMessages(): MessageSubscription {
    const abortController = new AbortController()
    const stream = this.client.subscribeMessages({}, {
      signal: abortController.signal,
    })
    return new MessageSubscription(stream, abortController)
  }
}