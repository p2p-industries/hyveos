import type { Client, Transport } from 'npm:@connectrpc/connect@2.0.1'
import {
  type RecvRequest,
  ReqResp as Service,
  type TopicQuery,
} from './gen/script_pb.ts'
import { AbortOnDispose, BaseService } from './core.ts'

export interface IncomingRequest {
  data: Uint8Array
  peerId: string
  topic: string | null
}

/**
 * Handle to an incoming request that allows responding to it.
 *
 * @example
 * ```ts
 *  const stream = client.reqresp.recv('my-topic')
 *  for await (const req of stream) {
 * 	 try {
 *  		const data = await processRequest(req)
 *  		await req.respond(data)
 *  	} catch (err) {
 *  		await req.error(err)
 *  	}
 *  }
 *  ```
 */
export class IncomingRequestHandle implements IncomingRequest {
  private client: Client<typeof Service>
  private seq: bigint
  private consumed: boolean = false
  public readonly data: Uint8Array
  public readonly peerId: string
  public readonly topic: string | null

  private constructor(
    client: Client<typeof Service>,
    seq: bigint,
    data: Uint8Array,
    peerId: string,
    topic: string | null,
  ) {
    this.client = client
    this.seq = seq
    this.data = data
    this.peerId = peerId
    this.topic = topic
  }

  /**
   * @private Internal method to create a new IncomingRequestHandle.
   * @ignore
   */
  public static __create(
    client: Client<typeof Service>,
    seq: bigint,
    data: Uint8Array,
    peerId: string,
    topic: string | null,
  ): IncomingRequestHandle {
    return new IncomingRequestHandle(client, seq, data, peerId, topic)
  }

  private consume() {
    if (this.consumed) {
      throw new Error('Request already consumed')
    }
    this.consumed = true
  }

  /**
   * Respond to the request with an error message because the processing failed.
   *
   * @param error The error message.
   * @throws If the request has already been responded to or the grpc call fails.
   * @returns A promise that resolves when the response has been sent.
   */
  public async error(error: Error | string) {
    const message = error instanceof Error ? error.message : error
    try {
      await this.client.respond({
        seq: this.seq,
        response: {
          response: {
            value: message,
            case: 'error',
          },
        },
      })
    } finally {
      this.consume()
    }
  }

  /**
   * Respond to the request with data.
   *
   * @param data The data to send in the response.
   * @throws If the request has already been responded to or the grpc call fails.
   * @returns A promise that resolves when the response has been sent.
   */
  public async respond(data: Uint8Array) {
    try {
      await this.client.respond({
        seq: this.seq,
        response: {
          response: {
            value: {
              data,
            },
            case: 'data',
          },
        },
      })
    } finally {
      this.consume()
    }
  }
}

export type HandlerIncomingRequest = Pick<
  IncomingRequestHandle,
  'data' | 'topic' | 'peerId'
>

/** A subscription to incoming requests.
 * @implements {AsyncIterable<IncomingRequestHandle>}
 */
export class ReqResSubscription extends AbortOnDispose
  implements AsyncIterable<IncomingRequestHandle> {
  private stream: AsyncIterable<RecvRequest>
  private client: Client<typeof Service>

  constructor(
    stream: AsyncIterable<RecvRequest>,
    client: Client<typeof Service>,
    abortController: AbortController,
  ) {
    super(abortController)
    this.stream = stream
    this.client = client
  }

  async *[Symbol.asyncIterator](): AsyncIterator<IncomingRequestHandle> {
    for await (const { msg, seq, peer } of this.stream) {
      if (!peer) {
        throw new Error('Missing peer in message')
      }
      if (!msg) {
        throw new Error('Missing message in message')
      }
      if (!msg.data) {
        throw new Error('Missing data in message')
      }
      const topic = msg.topic?.topic?.topic || null
      const { peerId } = peer
      const { data } = msg.data
      yield IncomingRequestHandle.__create(
        this.client,
        seq,
        data,
        peerId,
        topic,
      )
    }
  }
}

/**
 * A service for request response communication.
 */
export class ReqRes extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): ReqRes {
    return new ReqRes(Service, transport)
  }

  /**
   * Send a request to a peer and wait for a response.
   *
   * @param peerId The peer to send the request to.
   * @param data The data to send in the request.
   * @returns The response data.
   * @throws If the response is an error.
   */
  public async request(
    peerId: string,
    data: Uint8Array | string,
  ): Promise<Uint8Array> {
    if (typeof data === 'string') {
      data = new TextEncoder().encode(data)
    }
    const { response } = await this.client.send({
      peer: {
        peerId,
      },
      msg: {
        data: {
          data,
        },
      },
    })
    switch (response.case) {
      case 'data':
        return response.value.data
      case 'error':
        throw new Error(`Response with error: ${response.value}`)
      default:
        throw new Error(`Unknown response case: ${response}`)
    }
  }

  private static filterToQuery(filter: string | RegExp): TopicQuery {
    if (typeof filter === 'string') {
      return {
        query: {
          case: 'topic',
          value: {
            topic: filter,
            $typeName: 'script.Topic',
          },
        },
        $typeName: 'script.TopicQuery',
      }
    } else {
      return {
        query: {
          case: 'regex',
          value: filter.source,
        },
        $typeName: 'script.TopicQuery',
      }
    }
  }

  /**
   * Subscribe to incoming requests.
   *
   * @param filter The filter to apply to incoming requests. Can be a string or a RegExp.
   * @returns A subscription to incoming requests.
   *
   * @example
   * ```ts
   * const stream = client.reqresp.recv('my-topic')
   * for await (const req of stream) {
   *   console.log(req.data)
   * }
   * ```
   */
  public recv(filter: string | RegExp): ReqResSubscription {
    const abortController = new AbortController()
    const stream = this.client.recv(
      {
        query: ReqRes.filterToQuery(filter),
      },
      {
        signal: abortController.signal,
      },
    )
    return new ReqResSubscription(stream, this.client, abortController)
  }

  /**
   * Handle incoming requests.
   *
   * This is a convenience method over `ReqRes.recv` that allows you to handle incoming requests with a single function.
   *
   * @param filter The filter to apply to incoming requests. Can be a string or a RegExp.
   * @param handler The handler for incoming requests.
   *
   * @example
   * ```ts
   * client.reqresp.handle('my-topic', async (req) => {
   *   return new TextEncoder().encode('Hello World')
   * })
   * ```
   */
  public async handle(
    filter: string | RegExp,
    handler: (req: HandlerIncomingRequest) => Promise<Uint8Array>,
  ) {
    using stream = this.recv(filter)
    for await (const req of stream) {
      handler(req)
        .then(async (data) => {
          await req.respond(data)
        })
        .catch(async (err) => {
          if (err instanceof Error) {
            await req.error(err.message)
          } else if (typeof err === 'string') {
            await req.error(err)
          } else {
            await req.error('Unknown error')
          }
        })
    }
  }
}
