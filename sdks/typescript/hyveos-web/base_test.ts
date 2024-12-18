import { Client } from 'hyveos-sdk'
import { Connection } from './src/index.ts'
import { assertEquals } from 'jsr:@std/assert'

const connection = new Connection('http://localhost:8080')
const client = new Client(connection)

Deno.test('localdb-put-get', async () => {
  const random_key = Math.random().toString(36).substring(7)
  const random_value = Math.random().toString(36).substring(7)
  const random_value_encoded = new TextEncoder().encode(random_value)
  await client.localdb.put(random_key, random_value_encoded)
  const value = await client.localdb.get(random_key)
  assertEquals(value, random_value_encoded)
})
