import type { EventData, Graph, TopologyUpdateData } from '$lib/types';

function addVertex(g: Graph, peerId: string) {
  if (g.nodes.find((node) => node.peerId == peerId) == undefined) {
    g.nodes.push({ peerId: peerId, label: peerId });
  }
}

function addEdge(g: Graph, src: string, target: string) {
  console.log('Adding edges');

  addVertex(g, src);
  addVertex(g, target);

  g.links.push({ source: src, target: target });
}

function removeEdge(g: Graph, src: string, target: string) {
  g.links = g.links.filter((link) => link.source != src || link.target != target);
}

export function updateGraph(g: Graph, data: TopologyUpdateData) {
  if (data.events) {
    data.events.forEach((eventData: EventData) => {
      eventData.events.forEach((event) => {
        if (event.eventType == 'discovered' || event.eventType == 'init') {
          event.peerId.forEach((target) => addEdge(g, eventData.peerId, target));
        } else if (event.eventType == 'lost') {
          event.peerId.forEach((target) => removeEdge(g, eventData.peerId, target));
        }
      });
    });
  }
}

export function resetGraph(g: Graph) {
  g.nodes = [];
  g.links = [];
}
