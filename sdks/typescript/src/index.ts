import { createClient } from "@connectrpc/connect";
import { ReqResp } from "./gen/script_pb";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { createGrpcWebTransport } from "@connectrpc/connect-web";

export class Connection {
  constructor(uri: string) {
    if (typeof window === "undefined") {
      createGrpcWebTransport({
        baseUrl: uri,
      });
    }
  }
}
