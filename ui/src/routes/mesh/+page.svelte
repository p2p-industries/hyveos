<script lang="ts">
  import MeshGraph from '$lib/components/MeshGraph.svelte';
  import DataViewer from '$lib/components/DataViewer.svelte';
  import type { Message } from '$lib/types';
  import { onMount } from 'svelte';
  import { load, nodes, links, peerIdToLabel } from './mesh.svelte';
  import { Client } from 'hyveos-sdk';
  import { Connection } from 'hyveos-web';

  let graph = $derived({ nodes, links });

  let messages: Message[] = $state([]);

  onMount(async () => {
    try {
      const client = new Client(new Connection('http://localhost:4000'));
      load(client);

      const stream = client.debug.subscribeMessages();
      for await (const { event, sender } of stream) {
        if (event.case === 'gos') {
          const decoder = new TextDecoder('utf8');
          messages.push({
            sender,
            data: {
              type: 'gossipsub',
              topic: event.topic,
              message: btoa(decoder.decode(event.data))
            }
          });
        } else if (event.case === 'req') {
          const { data, topic, id, receiver } = event;
          if (!topic) continue;
          messages.push({
            sender,
            data: {
              type: 'req_resp',
              id,
              topic,
              data,
              receiver
            }
          });
        } else if (event.case === 'res') {
          const { id, response } = event;
          const messageIdx = messages.findIndex(({ data }) => {
            return data.type === 'req_resp' && data.id === id;
          });
          if (messageIdx !== -1) {
            if (messages[messageIdx].data.type === 'req_resp') {
              messages[messageIdx].data.response = response;
            }
          }
        }
      }
    } catch (e) {
      console.error(e);
    }
  });
</script>

<div class="flex flex-row h-full">
  <div class="grow">
    <MeshGraph {graph} />
  </div>
  <div id="messages" class="relative bg-gray-800 text-white h-full overflow-y-hidden">
    <ul class="absolute bottom-0 w-full p-4">
      {#each messages as message}
        {#if message.data.topic !== 'script/export_data'}
          <li class="rounded bg-gray-600 mb-2 p-2">
            <p class="text-xs font-mono">
              From <span class="text-rose-500">{peerIdToLabel(message.sender)}</span>
              {#if message.data.type === 'req_resp'}
                to <span class="text-rose-500">{peerIdToLabel(message.data.receiver)}</span>:
              {:else}
                on topic <span class="text-blue-500">'{message.data.topic}'</span>:
              {/if}
            </p>
            {#if message.data.type === 'req_resp'}
              {@const data = message.data.data}
              <div class="flex flex-row w-full">
                <p class="w-6">&rarr;</p>
                <p class="w-[calc(100%-1.5rem)] text-gray-400 break-words">
                  <DataViewer {data} />
                </p>
              </div>
              {#if message.data.response?.success}
                {@const responseData = message.data.response.data}
                <div class="flex flex-row w-full">
                  <p class="w-6">&larr;</p>
                  <p class="w-[calc(100%-1.5rem)] text-gray-400 break-words">
                    <DataViewer data={responseData} />
                  </p>
                </div>
              {/if}
            {:else}
              <p class="text-gray-400 text-pretty break-words">
                {message.data.message}
              </p>
            {/if}
          </li>
        {/if}
      {/each}
    </ul>
  </div>
</div>

<style>
  #messages {
    width: 400px;
  }
</style>
