import { Transport } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import type { ITransport } from 'hyveos-sdk';

export class Connection implements ITransport {
  private url: string;
  constructor(url: string) {
    this.url = url;
  }

  transport(): Transport {
    return createGrpcTransport({
      baseUrl: this.url
    });
  }
}
