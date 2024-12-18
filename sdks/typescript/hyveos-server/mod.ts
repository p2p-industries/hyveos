import type { Transport } from 'npm:@connectrpc/connect';
import { createGrpcTransport } from 'npm:@connectrpc/connect-node';
import type { ITransport } from 'hyveos-sdk';
import { connect } from 'node:net';

export class Connection implements ITransport {
  private url: string;
  constructor(url: string) {
    this.url = url;
  }

  transport(): Transport {
    if (this.url.startsWith('unix') || this.url.startsWith('http+unix')) {
      return createGrpcTransport({
        baseUrl: 'http://socket.localhost',
        nodeOptions: {
          createConnection: () => {
            const url = this.url.replace('http+unix://', '').replace('unix://', '');
            return connect(url);
          }
        }
      });
    }
    return createGrpcTransport({
      baseUrl: this.url
    });
  }
}
