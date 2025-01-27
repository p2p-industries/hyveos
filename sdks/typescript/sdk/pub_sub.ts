import type { Transport } from '@connectrpc/connect'
import { AbortOnDispose, BaseService, toBytes } from './core.ts'
import { PubSub as Service, type PubSubRecvMessage } from './gen/bridge_pb.ts'

export interface IncomingMessage<M = Uint8Array> {
  topic: string
  msg: M
  msgId: Uint8Array
  propagationSource: string
  source: string
}

/**
 * A subscription to a gossipsub topic.
 */
export class PubSubSubscription extends AbortOnDispose
  implements AsyncIterable<IncomingMessage>, Disposable {
  stream: AsyncIterable<PubSubRecvMessage>

  constructor(
    stream: AsyncIterable<PubSubRecvMessage>,
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
 * A handle to the pub-sub service.
 *
 * Exposes methods to interact with the pub-sub service,
 * like for subscribing to topics and publishing messages.
 */
export class PubSub extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): PubSub {
    return new PubSub(Service, transport)
  }

  /**
   * Subscribes to a gossipsub topic to receive messages sent to that topic.
   *
   * @param topic The topic to subscribe to.
   * @returns A subscription object that can be used to iterate over the messages sent to the topic.
   */
  public subscribe(topic: string): PubSubSubscription {
    const abortController = new AbortController()
    const stream = this.client.subscribe(
      {
        topic,
      },
      {
        signal: abortController.signal,
      },
    )
    return new PubSubSubscription(stream, abortController)
  }

  /**
   * Subscribes to a gossipsub topic to handle messages sent to that topic with a callback.
   *
   * @param topic The topic to subscribe to.
   * @param callback The callback to handle the messages sent to the topic.
   */
  public async handle(
    topic: string,
    callback: (msg: IncomingMessage) => Promise<void>,
  ) {
    using subscription = this.subscribe(topic)
    for await (const msg of subscription) {
      callback(msg).catch((err) => {
        console.error('Error handling message:', err)
      })
    }
  }

  /**
   * Subscribes to a gossipsub topic to handle messages with JSON-encoded data sent to that topic with a callback.
   *
   * @param topic The topic to subscribe to.
   * @param callback The callback to handle the messages sent to the topic with the data decoded from JSON.
   */
  public async handleJSON<M>(
    topic: string,
    callback: (msg: IncomingMessage<M>) => Promise<void>,
  ) {
    using subscription = this.subscribe(topic)
    for await (const msg of subscription) {
      callback({
        ...msg,
        msg: JSON.parse(new TextDecoder().decode(msg.msg)) as M,
      }).catch((err) => {
        console.error('Error handling message:', err)
      })
    }
  }

  /**
   * Publishes a message to a topic.
   *
   * @param topic The topic to publish the message to.
   * @param data The data of the message to publish.
   * @returns The ID of the message that was published.
   */
  public async publish(
    topic: string,
    data: Uint8Array | string,
  ): Promise<Uint8Array> {
    const { id } = await this.client.publish({
      topic: {
        topic,
      },
      data: {
        data: toBytes(data),
      },
    })
    return id
  }

  /**
   * Publishes a message with JSON-encoded data to a topic.
   *
   * @param topic The topic to publish the message to.
   * @param data The data of the message to publish, which will be JSON-encoded.
   * @returns The ID of the message that was published.
   */
  public async publishJSON<M>(
    topic: string,
    data: M,
  ): Promise<Uint8Array> {
    const { id } = await this.client.publish({
      topic: {
        topic,
      },
      data: {
        data: toBytes(JSON.stringify(data)),
      },
    })
    return id
  }
}
