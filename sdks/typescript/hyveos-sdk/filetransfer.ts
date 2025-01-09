import type { Transport } from 'npm:@connectrpc/connect'
import { BaseService, createJsonResult } from './core.ts'
import { FileTransfer as Service } from './gen/script_pb.ts'
import {
  array,
  integer,
  length,
  number,
  object,
  parse,
  pipe,
  string,
  ulid as ulidParse,
} from 'npm:valibot'

const uploadResponse = createJsonResult(object({
  id: pipe(string(), ulidParse()),
  hash: pipe(
    array(pipe(number(), integer('Number must be an integer'))),
    length(32),
  ),
}))

/**
 * A content identifier.
 *
 * Identifies a file on the network.
 */
export interface Cid {
  /** @ignore */
  id?: string
  /** @ignore */
  hash: Uint8Array
}

export class FileTransfer extends BaseService<typeof Service> {
  /** @ignore */
  private isUnix: boolean
  /** @ignore */
  private url: string

  /** @ignore */
  constructor(
    service: typeof Service,
    transport: Transport,
    isUnix: boolean,
    url: string,
  ) {
    super(service, transport)
    this.isUnix = isUnix
    this.url = url
  }

  /** @ignore */
  public static __create(
    transport: Transport,
    isUnix: boolean,
    url: string,
  ): FileTransfer {
    return new FileTransfer(Service, transport, isUnix, url)
  }

  /** @ignore */
  private async processResponse(response: Response): Promise<Cid> {
    const bodyJson = await response.json()
    const parsed = parse(uploadResponse, bodyJson)
    if (!parsed.success) {
      throw new Error(parsed.error)
    }
    const hash = new Uint8Array(32)
    for (let i = 0; i < 32; i++) {
      hash[i] = parsed.data.hash[i]
    }
    return {
      id: parsed.data.id,
      hash,
    }
  }

  /** @ignore */
  private async uploadFileFromBlob(
    blob: Blob | ReadableStream<Uint8Array> | Uint8Array,
    fileName: string,
  ): Promise<Cid> {
    const uploadUrl = `${this.url}/file-transfer/upload/${fileName}`

    const response = await fetch(uploadUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/octet-stream',
      },
      body: blob,
    })
    return await this.processResponse(response)
  }

  /**
   * Publish a file to the network.
   *
   * @param file The file to publish. If using unix sockets, this should be a file path. If using HTTP, this should be a file blob.
   * @param fileName The name of the file. Required when uploading a file blob.
   *
   * @returns The id and hash of the file.
   */
  public async publishFile(
    file: string | Uint8Array | Blob | ReadableStream<Uint8Array>,
    fileName?: string,
  ): Promise<Cid> {
    if (this.isUnix && typeof file !== 'string') {
      throw new Error(
        'This method is only available when not using unix sockets, you need to provide a file path',
      )
    }
    if (!this.isUnix && typeof file === 'string') {
      throw new Error(
        'This method is only available when using unix sockets, you need to provide a file blob',
      )
    }
    if (typeof file === 'string') {
      const { hash, id } = await this.client.publishFile({
        path: file,
      })
      return {
        hash,
        id: id?.ulid,
      }
    } else {
      if (!fileName) {
        throw new Error('fileName is required when uploading a file blob')
      }
      return this.uploadFileFromBlob(file, fileName)
    }
  }

  /**
   * Get a file from the network.
   *
   * This method is **only** available when using unix sockets.
   *
   * @param id The id of the file
   * @param hash The hash of the file
   *
   * @returns The path to the downloaded file
   */
  public async getFileLocally({ id, hash }: Cid): Promise<string> {
    if (!this.isUnix) {
      throw new Error('This method is only available when using unix sockets')
    }
    const { path } = await this.client.getFile({
      id: {
        ulid: id,
      },
      hash: hash,
    })
    return path
  }

  /**
   * Get a file from the network.
   *
   * This method is **only** available when **not** using unix sockets.
   *
   * @param id The id of the file
   * @param hash The hash of the file
   *
   * @returns A readable stream of the file
   */
  public async getFile({ id, hash }: Cid): Promise<ReadableStream> {
    if (this.isUnix) {
      throw new Error(
        'This method is only available when not using unix sockets',
      )
    }
    const url = new URL(`${this.url}/file-transfer/get-file`)
    if (id) {
      url.searchParams.set('id', id)
    }
    const hashString = `${hash.toString()}`
    url.searchParams.set('hash', hashString)
    const response = await fetch(url.toString())
    if (!response.ok) {
      throw new Error('Failed to get file')
    }
    if (!response.body) {
      throw new Error('Response body is empty')
    }
    return response.body
  }
}
