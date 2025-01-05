import type { ITransport } from "jsr:hyveos-sdk";
import { createGrpcWebTransport } from "npm:@connectrpc/connect-web";
import type { Transport } from "npm:@connectrpc/connect";

export class Connection implements ITransport {
  public url: string;

  constructor(url: string) {
    this.url = url;
  }

  transport(): Transport {
    return createGrpcWebTransport({
      baseUrl: this.url,
    });
  }

  isUnix(): boolean {
    return false;
  }
}
