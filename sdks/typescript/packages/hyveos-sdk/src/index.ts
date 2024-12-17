import { Transport } from '@connectrpc/connect';
import { ReqRes } from './reqresp';
import { GossipSub } from './gossipsub';
import { Discovery } from './discovery';
import { DHT } from './dht';
import { LocalDb } from './db';

export { ReqRes, GossipSub, Discovery, DHT, LocalDb };

export interface ITransport {
  transport(): Transport;
}

export class Client<T extends ITransport> {
  private transport: T;

  constructor(transport: T) {
    this.transport = transport;
  }

  public get reqresp(): ReqRes {
    return ReqRes.__create(this.transport.transport());
  }

  public get gossipsub(): GossipSub {
    return GossipSub.__create(this.transport.transport());
  }

  public get discovery(): Discovery {
    return Discovery.__create(this.transport.transport());
  }

  public get dht(): DHT {
    return DHT.__create(this.transport.transport());
  }

  public get localdb(): LocalDb {
    return LocalDb.__create(this.transport.transport());
  }
}
