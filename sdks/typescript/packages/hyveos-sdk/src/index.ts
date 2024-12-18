import type { Transport } from 'npm:@connectrpc/connect';
import { ReqRes } from './reqresp.ts';
import { GossipSub } from './gossipsub.ts';
import { Discovery } from './discovery.ts';
import { DHT } from './dht.ts';
import { LocalDb } from './db.ts';

export { DHT, Discovery, GossipSub, LocalDb, ReqRes };

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
