<script>
  import { createEventDispatcher, onMount } from "svelte";
  export let graph;
  const dispatch = createEventDispatcher();

  let draggingNode = null;
  let offset = { x: 0, y: 0 };
  let localGraph = { nodes: [], edges: [] };
  const gridSize = 40;
  let sourceForEdge = null;
  let svgElement;
  let viewBox = "0 0 1000 600";
  let zoomLevel = 1;

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
    e.stopPropagation();
  }

  function onPointerMove(e) {
    if (!draggingNode || !svgElement) return;
    let pt = svgElement.createSVGPoint();
    pt.x = e.clientX;
    pt.y = e.clientY;
    let cursorpt = pt.matrixTransform(svgElement.getScreenCTM().inverse());
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

  function onNodeClick(e, node) {
    // Shift-click to connect nodes
    if (e.shiftKey) {
      if (!sourceForEdge) {
        sourceForEdge = node.id;
      } else if (sourceForEdge !== node.id) {
        const exists = (localGraph.edges || []).some(ed => ed.from === sourceForEdge && ed.to === node.id);
        if (!exists) {
          localGraph.edges = [...(localGraph.edges || []), { from: sourceForEdge, to: node.id }];
          dispatch('updateGraph', { ...localGraph });
        }
        sourceForEdge = null;
      } else {
        sourceForEdge = null;
      }
    }
    // Normal click selects node
    if (!e.shiftKey) {
      dispatch('selectNode', { node });
    }
  }

  function statusColor(status) {
    switch (status) {
      case 'running': return 'var(--color-node-running)';
      case 'success': return 'var(--color-node-success)';
      case 'error': return 'var(--color-node-error)';
      case 'cache': return 'var(--color-node-cache)';
      default: return 'var(--color-node-pending)';
    }
  }

  function getNodeTypeColor(nodeType) {
    // Different colors for different node types
    const colors = {
      'Echo': '#6366f1', // indigo
      'FileReader': '#10b981', // emerald
      'FileCopier': '#f59e0b', // amber
      'Transcriber': '#8b5cf6', // violet
      'Summarizer': '#3b82f6', // blue
      'HttpRequest': '#ef4444', // red
      'VectorSearch': '#06b6d4', // cyan
    };
    return colors[nodeType] || colors['Echo'];
  }

  onMount(() => {
    // Initialize node positions if not set
    if (localGraph.nodes) {
      localGraph.nodes.forEach((node, i) => {
        if (node.x === undefined) node.x = 120 + (i * 160);
        if (node.y === undefined) node.y = 120;
      });
      dispatch('updateGraph', { ...localGraph });
    }
  });
</script>

<div class="graph-editor">
  <div class="graph-toolbar">
    <div class="toolbar-section">
      <span class="text-sm text-secondary">Workflow Graph</span>
    </div>
    <div class="toolbar-section">
      <span class="text-xs text-muted">Hold Shift + click to connect nodes • Drag to move</span>
    </div>
  </div>

  <svg 
    bind:this={svgElement}
    id="graph-svg" 
    width="100%" 
    height="500" 
    class="graph-svg"
    tabindex="0"
    {viewBox}
  >
    <!-- Background pattern -->
    <defs>
      <pattern id="grid" width={gridSize} height={gridSize} patternUnits="userSpaceOnUse">
        <path d="M {gridSize} 0 L 0 0 0 {gridSize}" fill="none" stroke="var(--color-border)" stroke-width="0.5" opacity="0.3"/>
      </pattern>
      
      <!-- Arrow marker for edges -->
      <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth" fill="var(--color-text-muted)">
        <path d="M0,0 L0,6 L9,3 z"/>
      </marker>
      
      <!-- Drop shadow filter -->
      <filter id="node-shadow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow dx="0" dy="2" stdDeviation="3" flood-opacity="0.1"/>
      </filter>
    </defs>
    
    <!-- Grid background -->
    <rect width="100%" height="100%" fill="url(#grid)" />

    <!-- Draw edges as smooth curves -->
    {#each renderedEdges as { edge, from, to } }
      <path 
        d={`M${from.x+80},${from.y+40} C${from.x+150},${from.y+40} ${to.x-50},${to.y+40} ${to.x},${to.y+40}`} 
        stroke="var(--color-text-muted)" 
        fill="none" 
        stroke-width="2" 
        marker-end="url(#arrow)"
        class="edge"
      />
    {/each}

    <!-- Draw nodes -->
    {#each localGraph.nodes as node}
      <g 
        transform={`translate(${node.x ?? 100},${node.y ?? 100})`} 
        class="node-group"
        class:node-selected={sourceForEdge === node.id}
        tabindex="0" 
        onclick={(e) => onNodeClick(e, node)}
      >
        <!-- Node shadow/background -->
        <rect 
          width="160" 
          height="80" 
          rx="12" 
          fill="var(--color-surface-elevated)" 
          stroke={sourceForEdge === node.id ? "var(--color-primary)" : "var(--color-border)"}
          stroke-width={sourceForEdge === node.id ? "2" : "1"}
          filter="url(#node-shadow)"
          class="node-bg"
        />
        
        <!-- Status indicator -->
        <rect 
          width="160" 
          height="4" 
          rx="2" 
          fill={statusColor(node.status)}
          class="node-status"
        />
        
        <!-- Node type indicator -->
        <circle 
          cx="20" 
          cy="25" 
          r="6" 
          fill={getNodeTypeColor(node.run)}
          class="node-type-indicator"
        />
        
        <!-- Node content -->
        <g class="node-content" onpointerdown={(e) => onNodePointerDown(e, node)}>
          <!-- Node ID -->
          <text x="35" y="25" class="node-id">{node.id}</text>
          
          <!-- Node type -->
          <text x="35" y="45" class="node-type">{node.run}</text>
          
          <!-- Status text -->
          <text x="35" y="65" class="node-status-text">{node.status || 'pending'}</text>
        </g>
        
        <!-- Connection points -->
        <circle cx="0" cy="40" r="4" fill="var(--color-primary)" class="connection-point connection-input" opacity="0.7"/>
        <circle cx="160" cy="40" r="4" fill="var(--color-primary)" class="connection-point connection-output" opacity="0.7"/>
      </g>
    {/each}
  </svg>
</div>

<style>
  .graph-editor {
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .graph-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    background: var(--color-surface-elevated);
    border-bottom: 1px solid var(--color-border);
  }

  .toolbar-section {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .graph-svg {
    background: var(--color-surface);
    cursor: grab;
  }

  .graph-svg:active {
    cursor: grabbing;
  }

  .graph-svg:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  .node-group {
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .node-group:hover .node-bg {
    stroke: var(--color-border-strong);
    filter: url(#node-shadow) drop-shadow(0 4px 8px rgba(0,0,0,0.1));
  }

  .node-group:hover .connection-point {
    opacity: 1;
  }

  .node-selected .node-bg {
    stroke: var(--color-primary);
    stroke-width: 2;
  }

  .node-content {
    cursor: move;
  }

  .node-id {
    font-family: var(--font-family-sans);
    font-size: 14px;
    font-weight: var(--font-semibold);
    fill: var(--color-text-primary);
  }

  .node-type {
    font-family: var(--font-family-sans);
    font-size: 12px;
    font-weight: var(--font-medium);
    fill: var(--color-text-secondary);
  }

  .node-status-text {
    font-family: var(--font-family-mono);
    font-size: 10px;
    font-weight: var(--font-normal);
    fill: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .connection-point {
    transition: opacity var(--transition-fast);
    cursor: crosshair;
  }

  .edge {
    transition: stroke var(--transition-fast);
  }

  .edge:hover {
    stroke: var(--color-primary);
    stroke-width: 3;
  }

  .node-bg {
    transition: all var(--transition-fast);
  }

  .node-status {
    transition: fill var(--transition-fast);
  }

  .node-type-indicator {
    transition: all var(--transition-fast);
  }
</style> 