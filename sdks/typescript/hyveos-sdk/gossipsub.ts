import type { Transport } from 'npm:@connectrpc/connect'
import { AbortOnDispose, BaseService } from './core.ts'
import {
  GossipSub as Service,
  type GossipSubRecvMessage,
} from './gen/script_pb.ts'

export interface IncomingMessage {
  topic: string
  msg: Uint8Array
  msgId: Uint8Array
  propagationSource: string
  source: string
}

/**
 * A subscription to a gossipsub topic.
 */
export class GossipsubSubscription extends AbortOnDispose
  implements AsyncIterable<IncomingMessage>, Disposable {
  stream: AsyncIterable<GossipSubRecvMessage>

  constructor(
    stream: AsyncIterable<GossipSubRecvMessage>,
    abortController: AbortController,
  ) {
    super(abortController)
    this.stream = stream
  }

  async *[Symbol.asyncIterator](): AsyncIterator<IncomingMessage> {
    for await (const { msg, msgId, propagationSource, source } of this.stream) {
      if (!msg?.data) throw new Error('missing data')
      if (!msg.topic?.topic) throw new Error('missing topic')
      if (!msgId?.id) throw new Error('missing id')
      if (!propagationSource) throw new Error('missing propagationSource')
      if (!source) throw new Error('missing source')

      yield {
        topic: msg.topic.topic,
        msg: msg.data.data,
        msgId: msgId.id,
        propagationSource: propagationSource.peerId,
        source: source.peerId,
      }
    }
  }
}

/**
 * A gossipsub client.
 */
export class GossipSub extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): GossipSub {
    return new GossipSub(Service, transport)
  }

  /**
   * Subscribe to a gossipsub topic to receive messages sent to that topic.
   *
   * @param topic The topic to subscribe to.
   * @returns A subscription object that can be used to iterate over messages.
   */
  public subscribe(topic: string): GossipsubSubscription {
    const abortController = new AbortController()
    const stream = this.client.subscribe(
      {
        topic,
      },
      {
        signal: abortController.signal,
      },
    )
    return new GossipsubSubscription(stream, abortController)
  }

  public async handle(
    topic: string,
    callback: (msg: IncomingMessage) => void,
  ): Promise<void> {
    using subscription = this.subscribe(topic)
    for await (const msg of subscription) {
      callback(msg)
    }
  }

  public async publish(
    topic: string,
    data: Uint8Array | string,
  ): Promise<Uint8Array> {
    if (typeof data === 'string') {
      data = new TextEncoder().encode(data)
    }
    const { id } = await this.client.publish({
      topic: {
        topic,
      },
      data: {
        data,
      },
    })
    return id
  }
}
