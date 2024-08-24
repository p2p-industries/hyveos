<script lang="ts">
  import MeshGraph from '$lib/components/MeshGraph.svelte';
  import type { Graph } from '$lib/types';
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

  type WsExportGraph = z.infer<typeof WsExportGraph>;

  let allPeerIds: Map<string, number> = new Map();

  let graph: Writable<Graph> = writable({
    nodes: [],
    links: []
  });

  onMount(() => {
    const websocket = new WebSocket('ws://localhost:3000/ws');

    websocket.addEventListener('open', () => {
      console.log('WebSocket connection established');
    });

    websocket.addEventListener('message', (event: MessageEvent) => {
      if (typeof event.data === 'string') {
        const data = JSON.parse(event.data);
        try {
          let exportGraph = WsExportGraph.parse(data);
          graph.set({
            nodes: exportGraph.nodes.map((node) => {
              const labelNumber =
                allPeerIds.get(node.peer_id) ||
                allPeerIds.set(node.peer_id, allPeerIds.size + 1).get(node.peer_id);
              const label = `Node ${labelNumber}`;
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

    return () => {
      websocket.close();
    };
  });
</script>

<MeshGraph {graph} />
