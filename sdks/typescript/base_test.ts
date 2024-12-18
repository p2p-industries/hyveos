import { Client } from 'hyveos-sdk';
import { Connection } from 'hyveos-server';
import { assertEquals } from 'jsr:@std/assert';

const connection = new Connection('unix:///tmp/hyved/running');
const client = new Client(connection);

Deno.test({
  name: 'localdb-put-get',
  async fn() {
    const random_key = Math.random().toString(36).substring(7);
    const random_value = Math.random().toString(36).substring(7);
    const random_value_encoded = new TextEncoder().encode(random_value);
    await client.localdb.put(random_key, random_value_encoded);
    const value = await client.localdb.get(random_key);
    assertEquals(value, random_value_encoded);
  },
  sanitizeOps: false,
  sanitizeResources: false
});
