import type { Client, Transport } from 'npm:@connectrpc/connect'
import { AbortOnDispose, BaseService, toBytes } from './core.ts'
import {
  type RecvRequest,
  ReqResp as Service,
  type TopicQuery,
} from './gen/bridge_pb.ts'

export interface IncomingRequest {
  data: Uint8Array
  peerId: string
  topic: string | null
}

/**
 * A handle that lets you respond to an inbound request.
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
   * Responds to this request with an error response.
   *
   * @param error The error message.
   * @throws If the request has already been responded to or the grpc call fails.
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
   * Responds to this request with a successful response.
   *
   * @param data The data to send in the response.
   * @throws If the request has already been responded to or the grpc call fails.
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

/**
 * A subscription to incoming requests.
 *
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
 * A handle to the request-response service.
 *
 * Exposes methods to interact with the request-response service,
 * like for sending and receiving requests, and for sending responses.
 */
export class ReqResp extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): ReqResp {
    return new ReqResp(Service, transport)
  }

  /**
   * Sends a request with an optional topic to a peer and returns the response.
   *
   * @param peerId The peer to send the request to.
   * @param data The data to send in the request.
   * @param topic The optional topic to send the request on.
   *     The peer must be subscribed to the topic to receive the request.
   *     If not provided, the peer must be subscribed to requests without a topic to receive the request.
   * @returns The response data.
   * @throws If the response is an error.
   */
  public async sendRequest(
    peerId: string,
    data: Uint8Array | string,
    topic?: string,
  ): Promise<Uint8Array> {
    const { response } = await this.client.send({
      peer: {
        peerId,
      },
      msg: {
        data: {
          data: toBytes(data),
        },
        topic: {
          topic: topic !== undefined ? { topic } : undefined,
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
            $typeName: 'bridge.Topic',
          },
        },
        $typeName: 'bridge.TopicQuery',
      }
    } else {
      return {
        query: {
          case: 'regex',
          value: filter.source,
        },
        $typeName: 'bridge.TopicQuery',
      }
    }
  }

  /**
   * Subscribes to a topic and returns a stream of incoming request handles.
   *
   * @param filter The filter to apply to incoming requests. Can be a string or a RegExp.
   *     If not provided, only requests without a topic are received.
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
  public recv(filter?: string | RegExp): ReqResSubscription {
    const abortController = new AbortController()
    const stream = this.client.recv(
      {
        query: filter !== undefined ? ReqResp.filterToQuery(filter) : undefined,
      },
      {
        signal: abortController.signal,
      },
    )
    return new ReqResSubscription(stream, this.client, abortController)
  }

  /**
   * Subscribes to a topic and handles incoming requests with a callback.
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
    handler: (req: HandlerIncomingRequest) => Promise<Uint8Array>,
    filter?: string | RegExp,
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
