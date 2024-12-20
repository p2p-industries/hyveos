import type { Client, Transport } from 'npm:@connectrpc/connect'
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

export class IncomingRequestHandle implements IncomingRequest {
  private client: Client<typeof Service>
  private seq: bigint
  public data: Uint8Array
  public peerId: string
  public topic: string | null

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

  public static __create(
    client: Client<typeof Service>,
    seq: bigint,
    data: Uint8Array,
    peerId: string,
    topic: string | null,
  ): IncomingRequestHandle {
    return new IncomingRequestHandle(client, seq, data, peerId, topic)
  }

  public async error(message: string) {
    await this.client.respond({
      seq: this.seq,
      response: {
        response: {
          value: message,
          case: 'error',
        },
      },
    })
  }

  public async respond(data: Uint8Array) {
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
  }
}

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

export class ReqRes extends BaseService<typeof Service> {
  public static __create(transport: Transport): ReqRes {
    return new ReqRes(Service, transport)
  }

  public async request(peerId: string, data: Uint8Array): Promise<Uint8Array> {
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

  public async handle(
    filter: string | RegExp,
    handler: (req: IncomingRequest) => Promise<Uint8Array>,
  ) {
    using stream = this.recv(filter)
    for await (const req of stream) {
      handler(req)
        .then(async (data) => {
          await req.respond(data)
        })
        .catch(async (err) => {
          await req.error(err.toString())
        })
    }
  }
}
