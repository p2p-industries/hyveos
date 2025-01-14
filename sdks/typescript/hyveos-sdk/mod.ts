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

import type { Transport } from 'npm:@connectrpc/connect'
import { Apps } from './apps.ts'
export { Apps } from './apps.ts'
import { Debug } from './debug.ts'
export { Debug } from './debug.ts'
import { Discovery } from './discovery.ts'
export { Discovery } from './discovery.ts'
import { FileTransfer } from './file_transfer.ts'
export { FileTransfer } from './file_transfer.ts'
import { KV } from './kv.ts'
export { KV } from './kv.ts'
import { LocalKV } from './local_kv.ts'
export { LocalKV } from './local_kv.ts'
import { Neighbours } from './neighbours.ts'
export { Neighbours } from './neighbours.ts'
import { PubSub } from './pub_sub.ts'
export { PubSub } from './pub_sub.ts'
import { ReqResp } from './req_resp.ts'
export { ReqResp } from './req_resp.ts'
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
   * @returns A handle to the application management service.
   */
  public get apps(): Apps {
    return Apps.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the debug service.
   */
  public get debug(): Debug {
    return Debug.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the discovery service.
   */
  public get discovery(): Discovery {
    return Discovery.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the file transfer service.
   */
  public get fileTransfer(): FileTransfer {
    return FileTransfer.__create(
      this.transport.transport(),
      this.transport.isUnix(),
      this.transport.url,
    )
  }

  /**
   * @returns A handle to the distributed key-value store service.
   */
  public get kv(): KV {
    return KV.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the local key-value store service.
   */
  public get localKV(): LocalKV {
    return LocalKV.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the neighbours service.
   */
  public get neighbours(): Neighbours {
    return Neighbours.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the pub-sub service.
   */
  public get pubSub(): PubSub {
    return PubSub.__create(this.transport.transport())
  }

  /**
   * @returns A handle to the request-response service.
   */
  public get reqResp(): ReqResp {
    return ReqResp.__create(this.transport.transport())
  }
}