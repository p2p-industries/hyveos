import type { Transport } from 'npm:@connectrpc/connect'
import { BaseService } from './core.ts'
import { DB as Service } from './gen/script_pb.ts'

/**
 * A client for the LocalDb service.
 */
export class LocalDb extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): LocalDb {
    return new LocalDb(Service, transport)
  }

  public async put(key: string, data: Uint8Array): Promise<Uint8Array | null> {
    const resp = await this.client.put({
      key,
      value: {
        data,
      },
    })
    return resp.data?.data ?? null
  }

  public async get(key: string): Promise<Uint8Array | null> {
    const resp = await this.client.get({ key })
    return resp.data?.data ?? null
  }
}
