import type { Transport } from 'npm:@connectrpc/connect'
import { AbortOnDispose, BaseService } from './core.ts'
import { DHT as Service, type Peer } from './gen/script_pb.ts'

/**
 * A stream of providers for a given key in a topic.
 *
 * @example
 *  To get the providers for a key in a topic, iterate over the stream:
 *  ```ts
 *  const stream = client.dht.getProviders('my-topic', key)
 *  for await (const peerId of stream) {
 *    console.log(peerId)
 *  }
 *  ```
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

/** A client for the DHT service. */
export class DHT extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): DHT {
    return new DHT(Service, transport)
  }

  /**
   * Put a record into the DHT.
   *
   * @param topic The topic to put the record into.
   * @param key The key of the record.
   * @param data The data of the record.
   * @returns A promise that resolves when the record is put.
   *
   * @example Put a record into the DHT
   * ```ts
   * await client.dht.putRecord('my-topic', key, data)
   * ```
   */
  public async putRecord(
    topic: string,
    key: Uint8Array,
    data: Uint8Array,
  ): Promise<void> {
    await this.client.putRecord({
      key: {
        key,
        topic: {
          topic,
        },
      },
      value: {
        data,
      },
    })
  }

  /**
   * Get a record from the DHT.
   *
   * @param topic The topic to get the record from.
   * @param key The key of the record.
   *
   * @returns A promise that resolves with the data of the record or null if the record is not found.
   *
   * @example Get a record from the DHT
   * ```ts
   * const data = await client.dht.getRecord('my-topic', key)
   * if (data) {
   *    console.log('Record found:', data)
   *  } else {
   *    console.log('Record not found')
   *  }
   *  ```
   */
  public async getRecord(
    topic: string,
    key: Uint8Array,
  ): Promise<Uint8Array | null> {
    const { data } = await this.client.getRecord({
      key,
      topic: {
        topic,
      },
    })
    if (!data) {
      return null
    }
    return data.data
  }

  /**
   * Remove a record from the DHT.
   *
   * This only affects the local node and will only take effect in the network once the record expires.
   *
   * @param topic The topic to remove the record from.
   * @param key The key of the record.
   */
  public async removeRecord(
    topic: string,
    key: Uint8Array,
  ): Promise<void> {
    await this.client.removeRecord({
      key,
      topic: {
        topic,
      },
    })
  }

  /**
   * Register as a provider for a given key in a topic.
   *
   * @param topic The topic to provide the key in.
   * @param key The key to provide.
   * @returns A promise that resolves when the key is provided.
   */
  public async provide(topic: string, key: Uint8Array): Promise<void> {
    await this.client.provide({
      topic: {
        topic,
      },
      key,
    })
  }

  /**
   * Get the providers for a given key in a topic.
   *
   * @param topic The topic to get the providers from.
   * @param key The key to get the providers for.
   * @returns A stream of providers for the key in the topic.
   */
  public getProviders(topic: string, key: Uint8Array): ProvidersStream {
    const abortController = new AbortController()
    const stream = this.client.getProviders(
      {
        topic: {
          topic,
        },
        key,
      },
      {
        signal: abortController.signal,
      },
    )
    return new ProvidersStream(stream, abortController)
  }

  /**
   * Stop providing a key in a topic.
   *
   * This only affects the local node and will only take effect in the network once the record expires.
   *
   * @param topic The topic to stop providing the key in.
   * @param key The key to stop providing.
   */
  public async stopProviding(topic: string, key: Uint8Array): Promise<void> {
    await this.client.stopProviding({
      key,
      topic: {
        topic,
      },
    })
  }
}
