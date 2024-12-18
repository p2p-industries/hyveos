import type { Transport } from 'npm:@connectrpc/connect';
import { AbortOnDispose, BaseService } from './core.ts';
import { Discovery as Service, type NeighbourEvent } from './gen/script_pb.ts';

export type Event =
  | {
      case: 'init';
      peers: string[];
    }
  | {
      case: 'discovered';
      peer: string;
    }
  | {
      case: 'lost';
      peer: string;
    };

export class DiscoverySubscription
  extends AbortOnDispose
  implements AsyncIterable<Event>, Disposable
{
  stream: AsyncIterable<NeighbourEvent>;

  constructor(stream: AsyncIterable<NeighbourEvent>, abotController: AbortController) {
    super(abotController);
    this.stream = stream;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<Event> {
    for await (const { event } of this.stream) {
      switch (event.case) {
        case 'init': {
          const peers = event.value.peers.map((peer) => peer.peerId);
          yield { case: 'init', peers };
          break;
        }
        case 'discovered':
          yield { case: 'discovered', peer: event.value.peerId };
          break;
        case 'lost':
          yield { case: 'lost', peer: event.value.peerId };
          break;
      }
    }
  }
}

export class Discovery extends BaseService<typeof Service> {
  public static __create(transport: Transport): Discovery {
    return new Discovery(Service, transport);
  }

  public async getOwnId(): Promise<string> {
    const { peerId } = await this.client.getOwnId({});
    return peerId;
  }

  public subscribe() {
    const abortController = new AbortController();
    const stream = this.client.subscribeEvents(
      {},
      {
        signal: abortController.signal
      }
    );
    return new DiscoverySubscription(stream, abortController);
  }
}
