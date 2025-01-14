import type { Transport } from 'npm:@connectrpc/connect@2.0.1'
import { AbortOnDispose, BaseService } from './core.ts'
import { Discovery as Service, type NeighbourEvent } from './gen/script_pb.ts'

export interface InitEvent {
  case: 'init'
  peers: string[]
}

export interface DiscoveredEvent {
  case: 'discovered'
  peer: string
}

export interface LostEvent {
  case: 'lost'
  peer: string
}

export type Event =
  | InitEvent
  | DiscoveredEvent
  | LostEvent

export class DiscoverySubscription extends AbortOnDispose
  implements AsyncIterable<Event>, Disposable {
  stream: AsyncIterable<NeighbourEvent>

  constructor(
    stream: AsyncIterable<NeighbourEvent>,
    abotController: AbortController,
  ) {
    super(abotController)
    this.stream = stream
  }

  async *[Symbol.asyncIterator](): AsyncIterator<Event> {
    for await (const { event } of this.stream) {
      switch (event.case) {
        case 'init': {
          const peers = event.value.peers.map((peer) => peer.peerId)
          yield { case: 'init', peers }
          break
        }
        case 'discovered':
          yield { case: 'discovered', peer: event.value.peerId }
          break
        case 'lost':
          yield { case: 'lost', peer: event.value.peerId }
          break
      }
    }
  }
}

/** A client for the Discovery service. */
export class Discovery extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): Discovery {
    return new Discovery(Service, transport)
  }

  /**
   * Get the own peer ID.
   *
   * @returns A promise that resolves to the own peer ID.
   */
  public async getOwnId(): Promise<string> {
    const { peerId } = await this.client.getOwnId({})
    return peerId
  }

  /**
   * Subscribe to the discovery service to receive neighbour events.
   *
   * @returns A subscription object that can be used to iterate over events.
   */
  public subscribe(): DiscoverySubscription {
    const abortController = new AbortController()
    const stream = this.client.subscribeEvents(
      {},
      {
        signal: abortController.signal,
      },
    )
    return new DiscoverySubscription(stream, abortController)
  }
}
