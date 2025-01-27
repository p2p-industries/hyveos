import type { Transport } from 'npm:@connectrpc/connect@2.0.1'
import { AbortOnDispose, BaseService } from './core.ts'
import { type NeighbourEvent, Neighbours as Service } from './gen/bridge_pb.ts'

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

/**
 * A handle to the neighbours service.
 *
 * Exposes methods to interact with the neighbours service,
 * such as subscribing to neighbour events and getting the current set of neighbours.
 */
export class Neighbours extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): Neighbours {
    return new Neighbours(Service, transport)
  }

  /**
   * Subscribes to neighbour events.
   *
   * It is guaranteed that an `init` event will be emitted,
   * followed only by `discovered` and `lost` events.
   *
   * @returns A subscription object that can be used to iterate over events.
   *     The object will emit an event whenever the local runtime detects a change in the set of neighbours.
   */
  public subscribe(): DiscoverySubscription {
    const abortController = new AbortController()
    const stream = this.client.subscribe(
      {},
      {
        signal: abortController.signal,
      },
    )
    return new DiscoverySubscription(stream, abortController)
  }

  /**
   * Returns the peer IDs of the current neighbours.
   *
   * @returns The current set of neighbours.
   */
  public async get(): Promise<string[]> {
    const { peers } = await this.client.get({})
    return peers.map((peer) => peer.peerId)
  }
}
