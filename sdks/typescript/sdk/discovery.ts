import type { Transport } from 'npm:@connectrpc/connect@2.0.1'
import { AbortOnDispose, BaseService, toBytes } from './core.ts'
import { Discovery as Service, type Peer } from './gen/bridge_pb.ts'

/**
 * A stream of providers for a discovery key.
 *
 * @example
 * To get the providers for a discovery key, iterate over the stream:
 * ```ts
 * const stream = client.discovery.getProviders('my-topic', key)
 * for await (const peerId of stream) {
 *   console.log(peerId)
 * }
 * ```
 */
export class ProvidersStream extends AbortOnDispose
  implements AsyncIterable<string>, Disposable {
  private stream: AsyncIterable<Peer>

  constructor(stream: AsyncIterable<Peer>, abortController: AbortController) {
    super(abortController)
    this.stream = stream
  }

  async *[Symbol.asyncIterator](): AsyncIterator<string> {
    for await (const { peerId } of this.stream) {
      yield peerId
    }
  }
}

/**
 * A handle to the discovery service.
 *
 * Exposes methods to interact with the discovery service,
 * like for marking the local runtime as a provider for a discovery key
 * or getting the providers for a discovery key.
 */
export class Discovery extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): Discovery {
    return new Discovery(Service, transport)
  }

  /**
   * Marks the local runtime as a provider for a discovery key.
   *
   * @param topic The topic to provide the key in.
   * @param key The discovery key to provide.
   */
  public async provide(topic: string, key: Uint8Array | string) {
    await this.client.provide({
      topic: {
        topic,
      },
      key: toBytes(key),
    })
  }

  /**
   * Gets the providers for a discovery key.
   *
   * @param topic The topic to get the providers from.
   * @param key The key to get the providers for.
   * @returns A stream of providers for the key in the topic.
   */
  public getProviders(
    topic: string,
    key: Uint8Array | string,
  ): ProvidersStream {
    const abortController = new AbortController()
    const stream = this.client.getProviders(
      {
        topic: {
          topic,
        },
        key: toBytes(key),
      },
      {
        signal: abortController.signal,
      },
    )
    return new ProvidersStream(stream, abortController)
  }

  /**
   * Stops providing a discovery key.
   *
   * Only affects the local node and only affects the network once the record expires.
   *
   * @param topic The topic to stop providing the key in.
   * @param key The discovery key to stop providing.
   */
  public async stopProviding(
    topic: string,
    key: Uint8Array | string,
  ) {
    await this.client.stopProviding({
      topic: {
        topic,
      },
      key: toBytes(key),
    })
  }
}
