import type { Transport } from 'npm:@connectrpc/connect@2.0.1'
import { BaseService } from './core.ts'
import { Scripting as Service } from './gen/script_pb.ts'

/**
 * A script that can be deployed to a peer or locally
 */
export interface DockerScript {
  /**
   * The docker image to run (e.g. 'redis')
   */
  image: string
  /**
   * The ports to expose (e.g. [6379])
   *  @default []
   */
  ports?: number[]
  /**
   * If true, the image is assumed to be available locally
   * and will not be pulled from a registry
   *  @default false
   */
  local?: boolean
}

/**
 * A client for the Scripting service.
 */
export class Scripting extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): Scripting {
    return new Scripting(Service, transport)
  }

  /**
   * Get the id of the current script
   *
   * @returns The id of the current script
   */
  public async getOwnId(): Promise<string> {
    const { ulid } = await this.client.getOwnId({})
    return ulid
  }

  /**
   * Deploy a script to a peer or locally (to the current machine)
   *  @param script The script to deploy
   *  @param persistent Whether the script should be persistent
   *  @param [peer] The peer to deploy the script to or undefined to deploy locally
   *
   *  @returns The id of the deployed script
   */
  public async deployScript(
    { image, ports, local }: DockerScript,
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

  /** List all running scripts
   * @param [peer] The peer to list the scripts from or undefined to list local scripts
   * @returns A list of running scripts
   */
  public async listRunningScripts(
    peer?: string,
  ): Promise<
    { image: string | undefined; name: string; id: string | undefined }[]
  > {
    const m = await this.client.listRunningScripts({
      peer: peer ? { peerId: peer } : undefined,
    })
    return m.scripts.map(({ image, name, id }) => {
      return { image: image?.name, name, id: id?.ulid }
    })
  }

  /**
   * Stop a running script by id on a peer or locally
   * @param id The id of the script to stop.
   * @param [peer] The peer to stop the script on or undefined to stop a local script
   * @returns A promise that resolves when the script has been stopped
   */
  public async stopScript(id: string, peer?: string): Promise<void> {
    await this.client.stopScript({
      id: { ulid: id },
      peer: peer ? { peerId: peer } : undefined,
    })
  }
}