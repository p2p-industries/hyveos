import { Client } from 'hyveos-sdk'
import { Connection } from 'hyveos-web'

const connection = new Connection('http://localhost:8080')
const client = new Client(connection)

Deno.test('dht-put-get', async () => {
  const randomKey = Math.random().toString(36).substring(7)
  const randomValue = Math.random().toString(36).substring(7)
  const encodedKey = new TextEncoder().encode(randomKey)
  const encodedValue = new TextEncoder().encode(randomValue)

  await client.dht.putRecord('test', encodedKey, encodedValue)
})
