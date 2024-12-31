/**
 * A platform independent library to interact with a HyveOS network.
 *
 * The transport depends on the platform. For the server, it is a grpc transport, for the web it is a grpc-web transport.
 * The transport is typically optained through the `hyveos-server` or `hyveos-web` package.
 * @module hyveos-sdk
 *
 * @example
 *  ```ts
 *  import { Client } from 'hyveos-sdk'
 *  import { Connection } from 'hyveos-web'
 *
 *  const transport = new Connection('http://localhost:8080')
 *  const client = new Client(transport)
 *  ```
 */
export default {}

import type { Transport } from 'npm:@connectrpc/connect'
import { ReqRes } from './reqresp.ts'
export * from './reqresp.ts'
import { GossipSub } from './gossipsub.ts'
export * from './gossipsub.ts'
import { Discovery } from './discovery.ts'
export * from './discovery.ts'
import { DHT } from './dht.ts'
export * from './dht.ts'
import { LocalDb } from './db.ts'
export * from './db.ts'
import { FileTransfer } from './filetransfer.ts'
export * from './filetransfer.ts'
import { Debug } from './debug.ts'
export * from './debug.ts'
import { Scripting } from './scripting.ts'
export * from './scripting.ts'

/**
 * Interface for the transport layer. Typically obtained through the `hyveos-server` or `hyveos-web` package.
 */
export interface ITransport {
  /**
   * @returns The transport object that is either a grpc-web or grpc transport.
   */
  transport(): Transport

  /**
   * @returns true if the transport is a unix socket, false if it is a regular http connection
   */
  isUnix(): boolean

  /** The base url of the transport. This is used for file transfer and largly irrelevant on the server */
  url: string
}

/**
 * The main client class that wraps all the services.
 * @example
 * ```ts
 * import { Client } from 'hyveos-sdk'
 * import { Connection } from 'hyveos-web'
 *
 * const transport = new Connection('http://localhost:8080')
 * const client = new Client(transport)
 * ```
 */
export class Client<T extends ITransport> {
  /**
   * @ignore
   */
  private transport: T

  /**
   * @param transport The transport object that is either a grpc-web or grpc transport.
   * @returns A new client object.
   */
  constructor(transport: T) {
    this.transport = transport
  }

  /**
   * get access to the reqresp service which is used for request response communication. The functionally is similar to http.
   *
   * @returns The ReqRes service.
   *
   * @example
   * ```ts
   * import { Client } from 'hyveos-sdk'
   * import { Connection } from 'hyveos-web'
   *
   * const transport = new Connection('http://localhost:8080')
   * const client = new Client(transport)
   * const response = await client.reqresp.request('Qansdasbd', (new TextEncoder()).encode('Hello World'))
   * const decoded = new TextDecoder().decode(response)
   * console.log(decoded)
   * ````
   */
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
