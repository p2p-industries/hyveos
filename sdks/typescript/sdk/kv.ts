import type { Transport } from '@connectrpc/connect'
import { BaseService, toBytes } from './core.ts'
import { KV as Service } from './gen/bridge_pb.ts'

/**
 * A handle to the distributed key-value store service.
 *
 * Exposes methods to interact with the key-value store service,
 * like for putting records into the key-value store or getting records from it.
 */
export class KV extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): KV {
    return new KV(Service, transport)
  }

  /**
   * Puts a record into the key-value store.
   *
   * @param topic The topic to put the record into.
   * @param key The key of the record.
   * @param value The value of the record.
   *
   * @example
   * Put a record into the DHT
   * ```ts
   * await client.dht.putRecord('my-topic', key, data)
   * ```
   */
  public async putRecord(
    topic: string,
    key: Uint8Array | string,
    value: Uint8Array | string,
  ) {
    await this.client.putRecord({
      key: {
        key: toBytes(key),
        topic: {
          topic,
        },
      },
      value: {
        data: toBytes(value),
      },
    })
  }

  /**
   * Puts a record with a JSON-encoded value into the key-value store.
   *
   * @param topic The topic to put the record into.
   * @param key The key of the record.
   * @param value The value of the record, which will be JSON-encoded.
   */
  public async putRecordJSON<V>(
    topic: string,
    key: Uint8Array | string,
    value: V,
  ) {
    await this.client.putRecord({
      key: {
        key: toBytes(key),
        topic: {
          topic,
        },
      },
      value: {
        data: toBytes(JSON.stringify(value)),
      },
    })
  }

  /**
   * Gets a record from the key-value store.
   *
   * @param topic The topic to get the record from.
   * @param key The key of the record.
   *
   * @returns The value of the record or null if the record is not found.
   *
   * @example
   * Get a record from the DHT
   * ```ts
   * const data = await client.dht.getRecord('my-topic', key)
   * if (data) {
   *   console.log('Record found:', data)
   * } else {
   *   console.log('Record not found')
   * }
   * ```
   */
  public async getRecord(
    topic: string,
    key: Uint8Array | string,
  ): Promise<Uint8Array | null> {
    const { data } = await this.client.getRecord({
      key: toBytes(key),
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
   * Gets a record with a JSON-encoded value from the key-value store.
   *
   * @param topic The topic to get the record from.
   * @param key The key of the record.
   *
   * @returns The value of the record decoded from JSON or null if the record is not found.
   */
  public async getRecordJSON<V>(
    topic: string,
    key: Uint8Array | string,
  ): Promise<V | null> {
    const { data } = await this.client.getRecord({
      key: toBytes(key),
      topic: {
        topic,
      },
    })
    if (!data) {
      return null
    }
    return JSON.parse(new TextDecoder().decode(data.data)) as V
  }

  /**
   * Removes a record from the key-value store.
   *
   * This only applies to the local node and only affects the network once the record expires.
   *
   * @param topic The topic to remove the record from.
   * @param key The key of the record.
   */
  public async removeRecord(
    topic: string,
    key: Uint8Array | string,
  ) {
    await this.client.removeRecord({
      key: toBytes(key),
      topic: {
        topic,
      },
    })
  }
}
