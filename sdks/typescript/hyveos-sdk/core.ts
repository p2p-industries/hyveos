import type { DescService } from 'npm:@bufbuild/protobuf'
import {
  type Client,
  createClient,
  type Transport,
} from 'npm:@connectrpc/connect'
import { literal, object, string, union } from 'jsr:@valibot/valibot'
import type { BaseIssue, BaseSchema } from 'jsr:@valibot/valibot'

export function createJsonResult<
  TInput,
  TOutput,
  TIssue extends BaseIssue<TInput>,
  T extends BaseSchema<TInput, TOutput, TIssue>,
>(data: T) {
  return union([
    object({
      success: literal(true),
      data,
    }),
    object({
      success: literal(false),
      error: string(),
    }),
  ])
}

export class BaseService<Service extends DescService> {
  protected client: Client<Service>

  constructor(service: Service, transport: Transport) {
    this.client = createClient(service, transport)
  }
}

export abstract class AbortOnDispose implements Disposable {
  private abortController: AbortController
  constructor(abortController: AbortController) {
    this.abortController = abortController
  }

  public cancel() {
    this.abortController.abort()
  }

  [Symbol.dispose](): void {
    this.cancel()
  }
}
