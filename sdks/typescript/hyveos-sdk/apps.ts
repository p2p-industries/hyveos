import type { Transport } from 'npm:@connectrpc/connect'
import { BaseService } from './core.ts'
import { Apps as Service, type RunningApp } from './gen/bridge_pb.ts'

/**
 * An application that can be deployed in the mesh.
 */
export interface DockerApp {
  /**
   * The docker image to run.
   *
   * Can contain a tag, e.g. `my-docker-image:latest`.
   */
  image: string
  /**
   * The ports to expose by the docker container running the application.
   * @default []
   */
  ports?: number[]
  /**
   * If true, the image is assumed to be available locally
   * and will not be pulled from a registry
   * @default false
   */
  local?: boolean
}

/**
 * A handle to the application management service.
 *
 * Exposes methods to interact with the application management service,
 * like for deploying and stopping apps on peers in the network.
 */
export class Apps extends BaseService<typeof Service> {
  /** @ignore */
  public static __create(transport: Transport): Apps {
    return new Apps(Service, transport)
  }

  /**
   * Deploys an application in a docker image to a peer in the network.
   *
   * @param peerId The peer ID to deploy the app to. If not provided, the app will be deployed on self.
   * @param persistent When this is set, the app will be restarted when the runtime is restarted.
   * @returns The ULID of the deployed application.
   */
  public async deploy(
    { image, ports, local }: DockerApp,
    peerId?: string,
    persistent: boolean = false,
  ): Promise<string> {
    const { ulid } = await this.client.deploy({
      app: {
        image: {
          name: image,
        },
        ports: ports ?? [],
      },
      local: local ?? false,
      peer: peerId !== undefined ? { peerId } : undefined,
      persistent,
    })
    return ulid
  }

  /**
   * Lists the running apps on a peer in the network.
   *
   * @param peerId The peer ID to list the apps of. If not provided, the app running on self will be listed.
   * @returns A list of running apps.
   */
  public async list_running(peerId?: string): Promise<RunningApp[]> {
    const { apps } = await this.client.listRunning({
      peer: peerId !== undefined ? { peerId } : undefined,
    })
    return apps
  }

  /**
   * Stops a running app with an ID on a peer in the network.
   *
   * @param appId The ULID of the app to stop.
   * @param peerId The peer ID to stop the app on. If not provided, the app will be stopped on self.
   */
  public async stop(appId: string, peerId?: string) {
    await this.client.stop({
      id: { ulid: appId },
      peer: peerId !== undefined ? { peerId } : undefined,
    })
  }

  /**
   * Get the ID of the current app.
   *
   * This can only be called from a running app.
   *
   * @returns The ULID of the current app.
   */
  public async getOwnAppId(): Promise<string> {
    const { ulid } = await this.client.getOwnAppId({})
    return ulid
  }
}
