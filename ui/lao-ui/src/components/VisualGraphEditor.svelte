<script>
  import { createEventDispatcher, onMount } from "svelte";
  export let graph;
  const dispatch = createEventDispatcher();

  let draggingNode = null;
  let offset = { x: 0, y: 0 };
  let localGraph = { nodes: [], edges: [] };
  const gridSize = 40;

  $: localGraph = graph ? JSON.parse(JSON.stringify(graph)) : { nodes: [], edges: [] };

  // Compute edges with from/to node references for rendering
  $: renderedEdges = (localGraph.edges || []).map(edge => {
    const from = (localGraph.nodes || []).find(n => n.id === edge.from);
    const to = (localGraph.nodes || []).find(n => n.id === edge.to);
    return from && to ? { edge, from, to } : null;
  }).filter(Boolean);

  function onNodePointerDown(e, node) {
    draggingNode = node;
    offset.x = e.offsetX - (node.x ?? 100);
    offset.y = e.offsetY - (node.y ?? 100);
    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
  }

  function onPointerMove(e) {
    if (!draggingNode) return;
    let svg = document.getElementById('graph-svg');
    if (!svg) return;
    let pt = svg.createSVGPoint();
    pt.x = e.clientX;
    pt.y = e.clientY;
    let cursorpt = pt.matrixTransform(svg.getScreenCTM().inverse());
    // Snap to grid
    draggingNode.x = Math.round((cursorpt.x - offset.x) / gridSize) * gridSize;
    draggingNode.y = Math.round((cursorpt.y - offset.y) / gridSize) * gridSize;
    dispatch('updateGraph', { ...localGraph });
  }

  function onPointerUp() {
    draggingNode = null;
    window.removeEventListener('pointermove', onPointerMove);
    window.removeEventListener('pointerup', onPointerUp);
    dispatch('updateGraph', { ...localGraph });
  }

  onMount(() => {
    // Initialize node positions if not set
    if (localGraph.nodes) {
      localGraph.nodes.forEach((node, i) => {
        if (node.x === undefined) node.x = 120 + (i * 120);
        if (node.y === undefined) node.y = 120;
      });
      dispatch('updateGraph', { ...localGraph });
    }
  });
</script>

<svg id="graph-svg" width="100%" height="400" style="background: #f8f8f8; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.08);" tabindex="0">
  <!-- Draw grid -->
  {#each Array(20) as _, i}
    <line x1={i*gridSize} y1="0" x2={i*gridSize} y2="400" stroke="#eee" />
  {/each}
  {#each Array(10) as _, j}
    <line x1="0" y1={j*gridSize} x2="1000" y2={j*gridSize} stroke="#eee" />
  {/each}

  <!-- Draw edges as smooth curves -->
  {#each renderedEdges as { edge, from, to } }
    <path d={`M${from.x+60},${from.y+30} C${from.x+120},${from.y+30} ${to.x-60},${to.y+30} ${to.x},${to.y+30}`} stroke="#888" fill="none" stroke-width="2" marker-end="url(#arrow)" />
  {/each}
  <defs>
    <marker id="arrow" markerWidth="10" markerHeight="10" refX="10" refY="5" markerUnits="strokeWidth">
      <path d="M0,0 L10,5 L0,10 z" fill="#888" />
    </marker>
  </defs>

  <!-- Draw nodes -->
  {#each localGraph.nodes as node}
    <g transform={`translate(${node.x ?? 100},${node.y ?? 100})`} tabindex="0">
      <rect width="120" height="60" rx="12" fill="#222" stroke="#444" stroke-width="2"
        on:pointerdown={(e) => onNodePointerDown(e, node)} />
      <text x="60" y="30" fill="#fff" font-size="18" text-anchor="middle" alignment-baseline="middle">{node.id}</text>
      <text x="60" y="48" fill="#aaa" font-size="14" text-anchor="middle" alignment-baseline="middle">{node.run}</text>
    </g>
  {/each}
</svg>

<style>
svg:focus {
  outline: 2px solid #396cd8;
}
</style> 