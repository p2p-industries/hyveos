import type { ITransport } from 'jsr:hyveos-sdk'
import { createGrpcWebTransport } from 'npm:@connectrpc/connect-web'

export class Connection implements ITransport {
  public url: string

  constructor(url: string) {
    this.url = url
  }

  transport() {
    return createGrpcWebTransport({
      baseUrl: this.url,
    })
  }

  isUnix(): boolean {
    return false
  }
}
