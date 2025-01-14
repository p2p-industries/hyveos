import type { Transport } from 'npm:@connectrpc/connect@2.0.1'
import { createGrpcTransport } from 'npm:@connectrpc/connect-node@2.0.1'
import type { ITransport } from 'jsr:@hyveos/sdk@0.1.0'
import { connect } from 'node:net'

export class Connection implements ITransport {
  public url: string
  constructor(url: string) {
    this.url = url
  }

  transport(): Transport {
    if (this.isUnix()) {
      return createGrpcTransport({
        baseUrl: 'http://socket.localhost',
        nodeOptions: {
          createConnection: () => {
            const url = this.url.replace('http+unix://', '').replace(
              'unix://',
              '',
            )
            return connect(url)
          },
        },
      })
    }
    return createGrpcTransport({
      baseUrl: this.url,
    })
  }

  isUnix(): boolean {
    return this.url.startsWith('unix') || this.url.startsWith('http+unix')
  }
}
