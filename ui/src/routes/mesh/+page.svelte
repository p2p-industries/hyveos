<script lang="ts">
  import MeshGraph from '$lib/components/MeshGraph.svelte';
  import type { Message } from '$lib/types';
  import { onMount } from 'svelte';
  // import { load, nodes, links, peerIdToLabel } from './mesh.svelte';
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
              <div class="flex flex-row w-full">
                <p class="w-6">&rarr;</p>
                <p class="w-[calc(100%-1.5rem)] text-gray-400 break-words">
                  {message.data.request}
                </p>
              </div>
              {#if message.data.response}
                <div class="flex flex-row w-full">
                  <p class="w-6">&larr;</p>
                  <p class="w-[calc(100%-1.5rem)] text-gray-400 break-words">
                    {message.data.response}
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
