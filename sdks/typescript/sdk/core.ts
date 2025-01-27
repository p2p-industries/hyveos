import type { DescService } from '@bufbuild/protobuf'
import { type Client, createClient, type Transport } from '@connectrpc/connect'
import { literal, object, string, union } from 'valibot'
import type { BaseIssue, BaseSchema } from 'valibot'

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

export function toBytes(data: Uint8Array | string): Uint8Array {
  if (typeof data === 'string') {
    return new TextEncoder().encode(data)
  }
  return data
}

export function repeatWithTimeoutCancellable(
  fn: () => Promise<void>,
  interval: number,
): () => void {
  let isActive = true
  ;(async () => {
    while (isActive) {
      await fn()
      await new Promise((resolve) => setTimeout(resolve, interval))
    }
  })()

  return () => {
    isActive = false
  }
}

export class BaseService<Service extends DescService> {
  protected client: Client<Service>

  constructor(service: Service, transport: Transport) {
    this.client = createClient(service, transport)
  }
}

/**
 * An abstract superclass to bridge the gap between the internal AbortController and the external Disposable interface.
 *
 * @example If you don't use TypeScript with version 5.2 or higher, use this class to free resources manually.
 *
 * ```ts
 *  const bar: AbortOnDispose = foo()
 *  try {
 *   // do something
 *  } finally {
 *   foo.cancel()
 *  }
 *  ```
 *
 *  @example If you use TypeScript with version 5.2 or higher, use the `using` keyword to have your item automatically disposed of.
 *  ```ts
 *  using bar: AbortOnDispose = foo()
 *
 *  // do something
 *  // bar is automatically disposed of here (at the end of the block)
 * ```
 */
export abstract class AbortOnDispose implements Disposable {
  private abortController: AbortController
  constructor(abortController: AbortController) {
    this.abortController = abortController
  }

  /**
   * Cancel the underlying subscription. Use this to free resources if you don't use TypeScript with version 5.2 or higher. (Otherwise, use the `using` keyword to have your item automatically disposed of.)
   */
  public cancel() {
    this.abortController.abort()
  }

  [Symbol.dispose](): void {
    this.cancel()
  }
}
