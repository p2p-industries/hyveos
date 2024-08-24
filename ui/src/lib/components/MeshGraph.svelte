<script lang="ts">
  import type { Readable } from 'svelte/store';
  import * as d3 from 'd3';
  import type { SimulationNodeDatum, SimulationLinkDatum } from 'd3';
  import type { Node, Graph } from '$lib/types';
  import { onMount } from 'svelte';

  export let graph: Readable<Graph>;

  type SimulationNode = Node & SimulationNodeDatum & { label: string };
  type SimulationLink = SimulationLinkDatum<SimulationNode> & { label: string };
  type SimulationGraph = { nodes: SimulationNode[]; links: SimulationLink[] };

  let svg: SVGSVGElement;
  let container: HTMLDivElement;

  let focusedNode: Node | null = null;

  class Simulator {
    private width: number;
    private height: number;
    private svgElement: d3.Selection<SVGSVGElement, unknown, null, undefined>;
    private simuGraph: SimulationGraph;
    private link: d3.Selection<SVGLineElement, SimulationLink, SVGGElement, unknown>;
    private node: d3.Selection<SVGGElement, SimulationNode, SVGGElement, unknown>;
    private simulation: d3.Simulation<SimulationNode, SimulationLink>;
    private firstUpdate: boolean = false;

    constructor(simuGraph: SimulationGraph) {
      this.simuGraph = simuGraph;
      const { width, height } = container.getBoundingClientRect();
      this.width = width;
      this.height = height;

      this.svgElement = d3.select(svg).attr('width', width).attr('height', height);
      this.link = this.svgElement.append('g').attr('class', 'links').selectAll('line');
      this.node = this.svgElement.append('g').attr('class', 'node').selectAll('g');
      this.simulation = d3
        .forceSimulation(this.simuGraph.nodes)
        .force(
          'link',
          d3
            .forceLink(this.simuGraph.links)
            .id((d) => {
              const node = d as SimulationNode;
              return node.peerId;
            })
            .distance(100)
        )
        .force('charge', d3.forceManyBody().strength(-725))
        .force('center', d3.forceCenter(this.width / 2, this.height / 2))
        .force('collision', d3.forceCollide().radius(50))
        .stop();
    }
    handleBackgroundClick() {
      if (focusedNode) {
        focusedNode = null;
        this.resetFocus();
      }
    }

    resetFocus() {
      this.svgElement.selectAll('rect').attr('opacity', 1);
      this.svgElement.selectAll('line').attr('opacity', 1);
    }

    simulate() {}

    ticked() {
      this.link
        .attr('x1', (d) => (d.source as SimulationNode).x!)
        .attr('y1', (d) => (d.source as SimulationNode).y!)
        .attr('x2', (d) => (d.target as SimulationNode).x!)
        .attr('y2', (d) => (d.target as SimulationNode).y!);
      this.node.attr('transform', (d) => `translate(${d.x}, ${d.y})`);
    }

    redrawGraph() {
      this.simulation.stop();
      this.simulation.alpha(1).restart();
    }

    update(newGraph: Graph) {
      this.simulation.stop();
      this.simulation.on('tick', null);
      this.updateData(newGraph);
      this.updateVis();
      this.simulation.on('tick', () => {
        this.ticked();
      });
      const alpha = this.firstUpdate ? 0.01 : 1;
      this.simulation.alpha(alpha).restart();
      this.firstUpdate = true;
    }

    updateData(newGraph: Graph) {
      const existingNodeIndex: Map<string, SimulationNode> = new Map(
        this.simuGraph.nodes.map((node) => [node.peerId, node])
      );
      const nodes: SimulationNode[] = [];
      for (const node of newGraph.nodes) {
        const existingNode = existingNodeIndex.get(node.peerId);
        if (existingNode) {
          nodes.push(existingNode);
        } else {
          const x = Math.random() * this.width;
          const y = Math.random() * this.height;
          nodes.push({ ...node, x, y });
        }
      }
      const existingLinksIndex: Map<string, SimulationLink> = new Map(
        this.simuGraph.links.map((link) => [link.label, link])
      );
      const links: SimulationLink[] = [];
      for (const link of newGraph.links) {
        const existingLink = existingLinksIndex.get(`${link.source}-${link.target}`);
        const { source, target } = link;
        if (existingLink) {
          links.push({
            ...existingLink,
            source,
            target,
            label: `${link.source}-${link.target}`
          });
        } else {
          links.push({
            source,
            target,
            label: `${link.source}-${link.target}`
          });
        }
      }
      this.simuGraph = { nodes, links };
    }

    updateVis() {
      this.link = this.link.data(
        this.simuGraph.links,
        (d) => `${(d.source as SimulationNode).peerId}-${(d.target as SimulationNode).peerId}`
      );
      this.link.exit().remove();
      this.link = this.link.enter().append('line').attr('class', 'link').merge(this.link);

      this.node = this.node.data(this.simuGraph.nodes, (d) => (d as SimulationNode).peerId);
      this.node.exit().remove();

      this.node = this.node.enter().append('g').merge(this.node);

      this.node.select('rect').remove();
      this.node
        .append('rect')
        .attr('x', -40)
        .attr('y', -30)
        .attr('width', 80)
        .attr('height', 60)
        .attr('rx', 10)
        .attr('ry', 10)
        .attr('fill', 'lightblue')
        .attr('stroke', '#000')
        .attr('stroke-width', 1);
      this.node
        .append('foreignObject')
        .attr('x', -40)
        .attr('y', -30)
        .attr('width', 80)
        .attr('height', 60)
        .html(
          (d) => `
                <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%;">
                    <div style="font-size: 12px;">${d.label}</div>
                </div>
            `
        );

      this.simulation.nodes(this.simuGraph.nodes);
      this.simulation
        .force<d3.ForceLink<SimulationNode, SimulationLink>>('link')!
        .links(this.simuGraph.links);
    }

    resize() {
      const { width, height } = container.getBoundingClientRect();
      this.width = width;
      this.height = height;
      this.svgElement
        .attr('width', width)
        .attr('height', height)
        .attr('viewBox', [0, 0, width, height]);
      this.simulation.force('center', d3.forceCenter(this.width / 2, this.height / 2));
      this.simulation.restart();
    }
  }

  let simulator: Simulator;

  onMount(() => {
    simulator = new Simulator({ nodes: [], links: [] });
    graph.subscribe((newGraph) => {
      simulator.update(newGraph);
    });
    container.addEventListener('click', () => {
      simulator.handleBackgroundClick();
    });
  });
</script>

<svelte:window
  on:resize={() => {
    if (simulator) {
      simulator.resize();
    }
  }}
/>

<div class="relative">
  <div bind:this={container} class="graph-container">
    <svg bind:this={svg}></svg>
  </div>
  <!-- A floating button -->
  <button
    class="btn absolute bottom-5 left-5 btn-accent"
    on:click={() => {
      if (simulator) {
        simulator.redrawGraph();
      }
    }}
  >
    Redraw Graph
  </button>
</div>

<style>
  .graph-container {
    width: 100%;
    height: 100vh; /* Adjust as needed for your layout */
  }

  svg {
    display: block;
    margin: auto;
  }
  :global(.node) {
    transition: transform 0.5s;
  }
  :global(.link) {
    stroke: #999;
    stroke-opacity: 0.6;
    stroke-width: 2px;
  }
</style>
