import type { Transport } from 'npm:@connectrpc/connect'
import { AbortOnDispose, BaseService } from './core.ts'
import { DHT as Service, type Peer } from './gen/script_pb.ts'

export class ProvidersStream extends AbortOnDispose
  implements AsyncIterable<string>, Disposable {
  stream: AsyncIterable<Peer>

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

export class DHT extends BaseService<typeof Service> {
  public static __create(transport: Transport): DHT {
    return new DHT(Service, transport)
  }

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

  public async provide(topic: string, key: Uint8Array): Promise<void> {
    await this.client.provide({
      topic: {
        topic,
      },
      key,
    })
  }

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
}
