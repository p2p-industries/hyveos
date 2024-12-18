import type { Transport } from 'npm:@connectrpc/connect'
import { createGrpcTransport } from 'npm:@connectrpc/connect-node'
import type { ITransport } from 'hyveos-sdk'

export class Connection implements ITransport {
  private url: string
  constructor(url: string) {
    this.url = url
  }

  transport(): Transport {
    return createGrpcTransport({
      baseUrl: this.url,
    })
  }
}
