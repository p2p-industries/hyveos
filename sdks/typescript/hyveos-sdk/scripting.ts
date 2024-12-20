import type { Transport } from 'npm:@connectrpc/connect'
import { BaseService } from './core.ts'
import { Scripting as Service } from './gen/script_pb.ts'

export interface DockerScript {
  image: string
  ports: number[]
}

export class Scripting extends BaseService<typeof Service> {
  public static __create(transport: Transport): Scripting {
    return new Scripting(Service, transport)
  }

  public async getOwnId() {
    const { ulid } = await this.client.getOwnId({})
    return ulid
  }

  private async deployScriptInner(
    { image, ports }: DockerScript,
    local: boolean,
    persistent: boolean,
    peer?: string,
  ): Promise<string> {
    const { ulid } = await this.client.deployScript({
      script: {
        image: {
          name: image,
        },
        ports,
      },
      local,
      peer: peer ? { peerId: peer } : undefined,
      persistent,
    })
    return ulid
  }

  public deployScript(
    script: DockerScript,
    local: boolean,
    persistent: boolean,
    peer: string,
  ) {
    return this.deployScriptInner(script, local, persistent, peer)
  }

  public deployScriptLocal(
    script: DockerScript,
    local: boolean,
    persistent: boolean,
  ) {
    return this.deployScriptInner(script, local, persistent)
  }

  private async listRunningScriptsInner(peer?: string) {
    const m = await this.client.listRunningScripts({
      peer: peer ? { peerId: peer } : undefined,
    })
    return m.scripts.map(({ image, name, id }) => {
      return { image: image?.name, name, id: id?.ulid }
    })
  }

  public listRunningScripts(peer: string) {
    return this.listRunningScriptsInner(peer)
  }

  public listRunningScriptsLocal() {
    return this.listRunningScriptsInner()
  }

  private async stopScriptInner(id: string, peer?: string) {
    await this.client.stopScript({
      id: { ulid: id },
      peer: peer ? { peerId: peer } : undefined,
    })
  }

  public stopScript(id: string, peer: string) {
    return this.stopScriptInner(id, peer)
  }

  public stopScriptLocal(id: string) {
    return this.stopScriptInner(id)
  }
}
