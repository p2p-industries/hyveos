/**
 * A platform independent library to interact with a HyveOS network.
 *
 * The transport depends on the platform. For the server, it is a grpc transport, for the web it is a grpc-web transport.
 * The transport is typically obtained through the `hyveos-server` or `hyveos-web` package.
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

import type { Transport } from 'npm:@connectrpc/connect@2.0.1'
import { ReqRes } from './reqresp.ts'
export { ReqRes } from './reqresp.ts'
import { GossipSub } from './gossipsub.ts'
export { GossipSub } from './gossipsub.ts'
import { Discovery } from './discovery.ts'
export { Discovery } from './discovery.ts'
import { DHT } from './dht.ts'
export { DHT } from './dht.ts'
import { LocalDb } from './db.ts'
export { LocalDb } from './db.ts'
import { FileTransfer } from './filetransfer.ts'
export { FileTransfer } from './filetransfer.ts'
import { Debug } from './debug.ts'
export { Debug } from './debug.ts'
import { Scripting } from './scripting.ts'
export { Scripting } from './scripting.ts'
export { AbortOnDispose } from './core.ts'

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
   * Get access to the reqresp service which is used for request response communication. The functionally is similar to http.
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

  /**
   * Get access to the gossipsub service which is used for pubsub communication.
   * @returns The GossipSub service.
   */
  public get gossipsub(): GossipSub {
    return GossipSub.__create(this.transport.transport())
  }

  /**
   * Get access to the discovery service which is used for peer discovery.
   * @returns The Discovery service.
   */
  public get discovery(): Discovery {
    return Discovery.__create(this.transport.transport())
  }

  /**
   * Get access to the dht service which is used for distributed hash table communication.
   * @returns The DHT service.
   */
  public get dht(): DHT {
    return DHT.__create(this.transport.transport())
  }

  /**
   * Get access to the localdb service which is used for local storage.
   * @returns The LocalDb service.
   */
  public get localdb(): LocalDb {
    return LocalDb.__create(this.transport.transport())
  }

  /**
   * Get access to the filetransfer service which is used for file transfer.
   * @returns The FileTransfer service.
   */
  public get filetransfer(): FileTransfer {
    return FileTransfer.__create(
      this.transport.transport(),
      this.transport.isUnix(),
      this.transport.url,
    )
  }

  /**
   * Get access to the debug service which is used for debugging.
   * @returns The Debug service.
   */
  public get debug(): Debug {
    return Debug.__create(this.transport.transport())
  }

  /**
   * Get access to the scripting service which is used for deploying and managing scripts.
   * @returns The Scripting service.
   */
  public get scripting(): Scripting {
    return Scripting.__create(this.transport.transport())
  }
}
