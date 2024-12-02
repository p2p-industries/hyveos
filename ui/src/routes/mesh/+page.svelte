<script lang="ts">
  import { page } from '$app/stores';
  import MeshGraph from '$lib/components/MeshGraph.svelte';
  import type { Graph, RequestResponse, Message } from '$lib/types';
  import { onMount } from 'svelte';
  import { writable, type Writable } from 'svelte/store';
  import { z } from 'zod';

  const WsNode = z.object({
    peer_id: z.string()
  });

  const WsLink = z.object({
    source: z.string(),
    target: z.string()
  });

  const WsExportGraph = z.object({
    nodes: z.array(WsNode),
    links: z.array(WsLink)
  });

  let allPeerIds: Map<string, number> = new Map();

  function getLabel(peerId: string): string {
    let labelNumber =
      allPeerIds.get(peerId) || allPeerIds.set(peerId, allPeerIds.size + 1).get(peerId);
    return `Node ${labelNumber}`;
  }

  let graph: Writable<Graph> = writable({
    nodes: [],
    links: []
  });

  let unansweredRequests: Map<string, number> = new Map();

  let messages: Message[] = $state([]);

  onMount(() => {
    const graphWsUrl = new URL($page.url.toString());
    graphWsUrl.protocol = graphWsUrl.protocol === 'https:' ? 'wss:' : 'ws:';
    graphWsUrl.pathname = '/ws-mesh-topology';
    const graphWebsocket = new WebSocket(graphWsUrl);

    graphWebsocket.addEventListener('open', () => {
      console.log('Graph WebSocket connection established');
    });

    graphWebsocket.addEventListener('message', (event: MessageEvent) => {
      if (typeof event.data === 'string') {
        const data = JSON.parse(event.data);
        try {
          let exportGraph = WsExportGraph.parse(data);
          graph.set({
            nodes: exportGraph.nodes.map((node) => {
              const label = getLabel(node.peer_id);
              return {
                peerId: node.peer_id,
                label
              };
            }),
            links: exportGraph.links.map(({ source, target }) => ({ source, target }))
          });
        } catch (e) {
          console.error('Invalid message received', e);
          return;
        }
      }
    });

    const messagesWsUrl = new URL($page.url.toString());
    messagesWsUrl.protocol = messagesWsUrl.protocol === 'https:' ? 'wss:' : 'ws:';
    messagesWsUrl.pathname = '/ws-messages';
    const messagesWebsocket = new WebSocket(messagesWsUrl);

    messagesWebsocket.addEventListener('open', () => {
      console.log('Messages WebSocket connection established');
    });

    messagesWebsocket.addEventListener('message', (event: MessageEvent) => {
      if (typeof event.data === 'string') {
        const data = JSON.parse(event.data);
        if (data.success === 'success') {
          let sender = data.sender as string;
          let event = data.event;
          if (event.Request) {
            let id = event.Request.id as string;
            let receiver = event.Request.receiver as string;
            let data = event.Request.msg.data as number[];
            let request = new TextDecoder().decode(new Uint8Array(data));
            let topic = event.Request.msg.topic as string | undefined;
            let message: Message = {
              sender,
              data: {
                type: 'req_resp',
                receiver,
                request,
                topic
              }
            };
            unansweredRequests.set(id, messages.length);
            messages = [...messages, message];
          } else if (event.Response) {
            let req_id = event.Response.req_id as string;
            let response: string;
            if (event.Response.response.Data) {
              let data = event.Response.response.Data as number[];
              response = new TextDecoder().decode(new Uint8Array(data));
            } else {
              let error = event.Response.response.Error as object;
              response = 'Error: ' + JSON.stringify(error);
            }
            let index = unansweredRequests.get(req_id);
            if (index !== undefined) {
              unansweredRequests.delete(req_id);
              let data = messages[index].data as RequestResponse;
              data.response = response;
              messages = [...messages];
            }
          } else if (event.GossipSub) {
            let topic = event.GossipSub.topic as string;
            let data = event.GossipSub.data as number[];
            let message = new TextDecoder().decode(new Uint8Array(data));
            let gossipsubMessage: Message = {
              sender,
              data: {
                type: 'gossipsub',
                topic,
                message
              }
            };
            messages = [...messages, gossipsubMessage];
          }
        } else {
          console.log(data);
        }
      }
    });

    return () => {
      graphWebsocket.close();
      messagesWebsocket.close();
    };
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
              From <span class="text-rose-500">{getLabel(message.sender)}</span>
              {#if message.data.type === 'req_resp'}
                to <span class="text-rose-500">{getLabel(message.data.receiver)}</span>:
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
