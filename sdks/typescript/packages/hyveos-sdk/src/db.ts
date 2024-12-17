import { Transport } from "@connectrpc/connect";
import { BaseService } from "./core";
import { DB as Service } from "./gen/script_pb";

export class LocalDb extends BaseService<typeof Service> {
  public static __create(transport: Transport) {
    return new LocalDb(Service, transport);
  }

  public async put(key: string, data: Uint8Array) {
    const resp = await this.client.put({
      key,
      value: {
        data,
      },
    });
    return resp.data?.data ?? null;
  }

  public async get(key: string) {
    const resp = await this.client.get({ key });
    return resp.data?.data ?? null;
  }
}
