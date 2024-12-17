import { Transport } from '@connectrpc/connect';
import { AbortOnDispose, BaseService } from './core';
import { Peer, DHT as Service } from './gen/script_pb';

export class ProvidersStream extends AbortOnDispose implements AsyncIterable<string>, Disposable {
  stream: AsyncIterable<Peer>;

  constructor(stream: AsyncIterable<Peer>, abortController: AbortController) {
    super(abortController);
    this.stream = stream;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<string> {
    for await (const { peerId } of this.stream) {
      yield peerId;
    }
  }
}

export class DHT extends BaseService<typeof Service> {
  public static __create(transport: Transport): DHT {
    return new DHT(Service, transport);
  }

  public async putRecord(key: Uint8Array, data: Uint8Array) {
    await this.client.putRecord({
      key: {
        key
      },
      value: {
        data
      }
    });
  }

  public async getRecord(key: Uint8Array) {
    const { data } = await this.client.getRecord({
      key
    });
    if (!data) {
      return null;
    }
    return data.data;
  }

  public async provide(key: Uint8Array) {
    await this.client.provide({
      key
    });
  }

  public getProviders(key: Uint8Array) {
    const abortController = new AbortController();
    const stream = this.client.getProviders({ key }, { signal: abortController.signal });
    return new ProvidersStream(stream, abortController);
  }
}
