import { DescService } from '@bufbuild/protobuf';
import { Client, createClient, Transport } from '@connectrpc/connect';

export class BaseService<Service extends DescService> {
  protected client: Client<Service>;

  constructor(service: Service, transport: Transport) {
    this.client = createClient(service, transport);
  }
}

export abstract class AbortOnDispose implements Disposable {
  private abortController: AbortController;
  constructor(abortController: AbortController) {
    this.abortController = abortController;
  }

  public cancel() {
    this.abortController.abort();
  }

  [Symbol.dispose](): void {
    this.cancel();
  }
}
