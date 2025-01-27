import type { ITransport } from 'jsr:@hyveos/sdk@0.1.0'
import { createGrpcWebTransport } from 'npm:@connectrpc/connect-web@2.0.1'
import type { Transport } from 'npm:@connectrpc/connect@2.0.1'

export class Connection implements ITransport {
  public url: string

  constructor(url: string) {
    this.url = url
  }

  transport(): Transport {
    return createGrpcWebTransport({
      baseUrl: this.url,
    })
  }

  isUnix(): boolean {
    return false
  }
}
