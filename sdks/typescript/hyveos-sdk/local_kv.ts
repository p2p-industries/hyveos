import type { Transport } from 'npm:@connectrpc/connect'
import { BaseService, toBytes } from './core.ts'
import { LocalKV as Service } from './gen/bridge_pb.ts'

/**
 * A handle to the local key-value store service.
 *
 * Exposes methods to interact with the key-value store service,
 * like putting and getting key-value records.
 * The key-value store is local to the runtime and is not shared with other runtimes.
 * However, it is persisted across restarts of the runtime.
 */
export class LocalKV extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): LocalKV {
    return new LocalKV(Service, transport)
  }

  /**
   * Puts a record into the key-value store.
   *
   * This only has local effects and does not affect other runtimes.
   * However, the record is persisted across restarts of the runtime.
   *
   * @param key The key of the record.
   * @param value The value of the record.
   * @returns The previous value of the record if it exists, or null otherwise.
   */
  public async put(
    key: string,
    value: Uint8Array | string,
  ): Promise<Uint8Array | null> {
    const resp = await this.client.put({
      key,
      value: {
        data: toBytes(value),
      },
    })
    return resp.data?.data ?? null
  }

  /**
   * Puts a record with a JSON-encoded value into the key-value store.
   *
   * This only has local effects and does not affect other runtimes.
   * However, the record is persisted across restarts of the runtime.
   *
   * @param key The key of the record.
   * @param value The value of the record, which will be JSON-encoded.
   * @returns The previous value of the record decoded from JSON if it exists, or null otherwise.
   */
  public async putJSON<V>(
    key: string,
    value: V,
  ): Promise<V | null> {
    const resp = await this.client.put({
      key,
      value: {
        data: toBytes(JSON.stringify(value)),
      },
    })
    const data = resp.data?.data
    if (!data) {
      return null
    }
    return JSON.parse(new TextDecoder().decode(data)) as V
  }

  /**
   * Gets a record from the key-value store if it exists.
   *
   * This will not return values from other runtimes.
   *
   * @param key The key of the record.
   * @returns The value of the record if it exists, or null otherwise.
   */
  public async get(key: string): Promise<Uint8Array | null> {
    const resp = await this.client.get({ key })
    return resp.data?.data ?? null
  }

  /**
   * Gets a record with a JSON-encoded value from the key-value store.
   *
   * @param key The key of the record.
   * @returns The value of the record decoded from JSON or null if the record is not found.
   */
  public async getJSON<V>(
    key: string,
  ): Promise<V | null> {
    const resp = await this.client.get({ key })
    const data = resp.data?.data
    if (!data) {
      return null
    }
    return JSON.parse(new TextDecoder().decode(data)) as V
  }
}
