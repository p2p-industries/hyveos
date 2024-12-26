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

interface Cid {
  id?: string
  hash: Uint8Array
}

export class FileTransfer extends BaseService<typeof Service> {
  private isUnix: boolean
  private url: string

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

  public static __create(
    transport: Transport,
    isUnix: boolean,
    url: string,
  ): FileTransfer {
    return new FileTransfer(Service, transport, isUnix, url)
  }

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

  private async uploadFileFromBlob(
    blob: Blob | ReadableStream,
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

  public async publishFile(
    file: string | Blob | ReadableStream,
    fileName?: string,
  ): Promise<Cid> {
    if (this.isUnix && typeof file === 'string') {
      const { hash, id } = await this.client.publishFile({
        path: file,
      })
      return {
        hash,
        id: id?.ulid,
      }
    } else if (
      !this.isUnix &&
      (file instanceof Blob || file instanceof ReadableStream) && fileName
    ) {
      return this.uploadFileFromBlob(file, fileName)
    } else {
      throw new Error('Invalid arguments')
    }
  }

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

  public async getFile({ id, hash }: Cid): Promise<ReadableStream> {
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
