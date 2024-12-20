import type { Transport } from 'npm:@connectrpc/connect'
import { ReqRes } from './reqresp.ts'
import { GossipSub } from './gossipsub.ts'
import { Discovery } from './discovery.ts'
import { DHT } from './dht.ts'
import { LocalDb } from './db.ts'
import { FileTransfer } from './filetransfer.ts'
import { Debug } from './debug.ts'
import { Scripting } from './scripting.ts'

export {
  Debug,
  DHT,
  Discovery,
  FileTransfer,
  GossipSub,
  LocalDb,
  ReqRes,
  Scripting,
}

export interface ITransport {
  transport(): Transport

  isUnix(): boolean

  url: string
}

export class Client<T extends ITransport> {
  private transport: T

  constructor(transport: T) {
    this.transport = transport
  }

  public get reqresp(): ReqRes {
    return ReqRes.__create(this.transport.transport())
  }

  public get gossipsub(): GossipSub {
    return GossipSub.__create(this.transport.transport())
  }

  public get discovery(): Discovery {
    return Discovery.__create(this.transport.transport())
  }

  public get dht(): DHT {
    return DHT.__create(this.transport.transport())
  }

  public get localdb(): LocalDb {
    return LocalDb.__create(this.transport.transport())
  }

  public get filetransfer(): FileTransfer {
    return FileTransfer.__create(
      this.transport.transport(),
      this.transport.isUnix(),
      this.transport.url,
    )
  }

  public get debug(): Debug {
    return Debug.__create(this.transport.transport())
  }

  public get scripting(): Scripting {
    return Scripting.__create(this.transport.transport())
  }
}
