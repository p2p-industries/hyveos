import type { Node } from "$lib/types";
import type { Link } from "$lib/types";
import type { Client } from "@hyveos/sdk";
import type { Connection } from "@hyveos/web";

export const nodes = $state<Node[]>([]);
export const links = $state<Link[]>([]);

const nodeToNumber: Map<string, string> = new Map();
const nodeSet = $derived(new Set(nodes));
const linkSet = $derived(new Set(links));

export const peerIdToLabel = (peerId: string) => {
  if (!nodeToNumber.has(peerId)) {
    nodeToNumber.set(peerId, `Node ${nodeToNumber.size}`);
  }
  return nodeToNumber.get(peerId)!;
};

export async function load(client: Client<Connection>) {
  const stream = client.debug.subscribeMeshTopology();

  for await (const { peer, neighbour } of stream) {
    if (neighbour.case === "init") {
      if (
        !nodeSet.has({
          peerId: peer,
          label: peerIdToLabel(peer),
        })
      ) {
        nodes.push({
          peerId: peer,
          label: peerIdToLabel(peer),
        });
      }
      for (const p of neighbour.peers) {
        if (
          !nodeSet.has({
            peerId: p,
            label: peerIdToLabel(p),
          })
        ) {
          nodes.push({
            peerId: p,
            label: peerIdToLabel(p),
          });
        }
        if (
          !linkSet.has({
            source: peerIdToLabel(peer),
            target: peerIdToLabel(p),
          })
        ) {
          links.push({
            source: peerIdToLabel(peer),
            target: peerIdToLabel(p),
          });
        }
      }
    } else if (neighbour.case === "lost") {
      const linkIdx = links.findIndex(
        (l) =>
          (l.source === peerIdToLabel(peer) &&
            l.target === peerIdToLabel(neighbour.peer)) ||
          (l.source === peerIdToLabel(neighbour.peer) &&
            l.target === peerIdToLabel(peer)),
      );
      if (linkIdx !== -1) {
        links.splice(linkIdx, 1);
      }
    } else if (neighbour.case === "discovered") {
      if (
        !linkSet.has({
          source: peerIdToLabel(peer),
          target: peerIdToLabel(neighbour.peer),
        })
      ) {
        links.push({
          source: peerIdToLabel(peer),
          target: peerIdToLabel(neighbour.peer),
        });
      }
    }
  }
}
