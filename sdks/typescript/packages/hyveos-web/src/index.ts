import type { ITransport } from "hyveos-sdk";
import { createGrpcWebTransport } from "@connectrpc/connect-web";

export class Connection implements ITransport {
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  transport() {
    return createGrpcWebTransport({
      baseUrl: this.url,
    });
  }
}
