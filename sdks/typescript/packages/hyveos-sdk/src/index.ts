import { createClient, Transport } from "@connectrpc/connect";
import { ReqRes } from "./reqresp";

export { ReqRes };

export interface ITransport {
  transport(): Transport;
}

export class Client<T extends ITransport> {
  private transport: T;

  constructor(transport: T) {
    this.transport = transport;
  }

  public get reqresp(): ReqRes {
    return ReqRes.__create(this.transport.transport());
  }
}
