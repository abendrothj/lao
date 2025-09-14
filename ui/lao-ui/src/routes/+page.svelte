<script>
  import { invoke } from "@tauri-apps/api/core";
  import { mkdir, writeTextFile } from "@tauri-apps/plugin-fs";
  import { dndzone } from "svelte-dnd-action";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import VisualGraphEditor from "../components/VisualGraphEditor.svelte";
  import Button from "../lib/components/Button.svelte";
  import Card from "../lib/components/Card.svelte";
  import Input from "../lib/components/Input.svelte";
  import Alert from "../lib/components/Alert.svelte";

  let name = $state("");
  let greetMsg = $state("");

  // FlowBuilder state
  let workflowPath = $state("");
  let graph = $state(null);
  let error = $state("");
  let newNodeName = $state("");
  let newNodeType = $state("Echo");
  let yamlExport = $state("");
  let newWorkflowFilename = $state("new_workflow.yaml");
  let plugins = $state([]);
  let selectedNode = $state(null);
  let yamlText = $state("");

  // Drag-and-drop state
  let dndNodes = $state([]);
  $effect(() => { if (graph) dndNodes = graph.nodes; });

  function handleDnd({ detail }) {
    if (graph) {
      graph.nodes = detail.items;
    }
  }

  // Live logs
  let liveLogs = $state([]);
  let unlistenStatus;
  let unlistenDone;
  let isRunning = $state(false);

  onMount(async () => {
    unlistenStatus = await listen("workflow:status", ({ payload }) => {
      liveLogs = [...liveLogs, JSON.stringify(payload)];
      if (liveLogs.length > 200) liveLogs.shift();
    });
    unlistenDone = await listen("workflow:done", ({ payload }) => {
      liveLogs = [...liveLogs, `DONE: ${JSON.stringify(payload)}`];
      isRunning = false;
    });
    try {
      const list = await invoke("list_plugins_for_ui");
      // Defensive: ensure array and non-empty; if empty, retry after short delay once (backend may still be initializing)
      if (Array.isArray(list) && list.length > 0) {
        plugins = list;
      } else {
        setTimeout(async () => {
          try {
            const list2 = await invoke("list_plugins_for_ui");
            if (Array.isArray(list2)) plugins = list2;
          } catch (err) {
            console.error("Plugin list retry failed", err);
          }
        }, 500);
      }
    } catch (e) {
      console.error("Failed to load plugins", e);
    }
    return () => { if (unlistenStatus) unlistenStatus(); if (unlistenDone) unlistenDone(); };
  });

  // Prompt-driven workflow state
  let prompt = $state("");
  let generatedYaml = $state("");
  let genError = $state("");
  let genGraph = $state(null);
  let isGenerating = $state(false);

  async function greet(event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }

  async function loadGraph() {
    error = "";
    try {
      graph = await invoke("get_workflow_graph", { path: workflowPath });
    } catch (e) {
      error = e.message || e;
      graph = null;
    }
  }

  function addNode() {
    if (!graph) return;
    const id = `step${graph.nodes.length + 1}`;
    graph.nodes = [
      ...graph.nodes,
      { id, run: newNodeType, input_type: null, output_type: null, status: "pending" }
    ];
    newNodeName = "";
    newNodeType = "Echo";
  }

  async function runWorkflow(parallel = false) {
    if (!workflowPath) { error = "Set a workflow path first"; return; }
    error = "";
    liveLogs = [];
    isRunning = true;
    try {
      await invoke("run_workflow_stream", { path: workflowPath, parallel });
    } catch (e) {
      error = e.message || e;
      isRunning = false;
    }
  }

  function removeNode(id) {
    if (!graph) return;
    graph.nodes = graph.nodes.filter(n => n.id !== id);
    graph.edges = graph.edges.filter(e => e.from !== id && e.to !== id);
  }

  function exportYAML() {
    if (!graph) return;
    let yaml = `workflow: "Visual Flow"
steps:
`;
    for (const node of graph.nodes) {
      yaml += `  - run: ${node.run}
`;
    }
    yamlExport = yaml;
    yamlText = yaml;
  }

  async function generateWorkflowFromPrompt() {
    genError = "";
    generatedYaml = "";
    genGraph = null;
    if (!prompt.trim()) {
      genError = "Please enter a prompt.";
      return;
    }
    isGenerating = true;
    try {
      const yaml = await invoke("dispatch_prompt", { prompt });
      generatedYaml = yaml;
      genError = "";
      // Parse YAML to graph for visualization
      // Save YAML to a temp file and load as graph
      // For now, call get_workflow_graph with a temp file
      const tempPath = "workflows/generated_from_prompt.yaml";
      await mkdir("workflows", { recursive: true });
      await writeTextFile(tempPath, yaml);
      genGraph = await invoke("get_workflow_graph", { path: tempPath });
    } catch (e) {
      genError = e.message || e;
    } finally {
      isGenerating = false;
    }
  }

  function newWorkflow() {
    graph = { nodes: [], edges: [] };
    error = "";
    workflowPath = "";
    yamlExport = "";
  }

  async function saveWorkflowAsYaml(filename) {
    if (!graph) return;
    let yaml = `workflow: "Visual Flow"\nsteps:\n`;
    for (const node of graph.nodes) {
      yaml += `  - run: ${node.run}\n`;
    }
    try {
      await writeTextFile(filename, yaml);
      yamlExport = `Saved to ${filename}`;
      yamlText = yaml;
    } catch (e) {
      yamlExport = `Error saving: ${e.message || e}`;
    }
  }

  function onSelectNode(e) {
    selectedNode = e.detail.node;
  }

  function updateSelectedNodeRun(newRun) {
    if (!graph || !selectedNode) return;
    const n = graph.nodes.find(n => n.id === selectedNode.id);
    if (n) n.run = newRun;
  }

  function removeSelectedNode() {
    if (!selectedNode) return;
    removeNode(selectedNode.id);
    selectedNode = null;
  }

  function importYAML() {
    try {
      const lines = yamlText.split(/\r?\n/);
      const runs = lines.filter(l => l.trim().startsWith("- run:")).map(l => l.split(":")[1].trim());
      graph = { nodes: runs.map((r, i) => ({ id: `step${i+1}`, run: r, status: "pending" })), edges: [] };
      error = "";
    } catch (e) {
      error = `Failed to import YAML: ${e.message || e}`;
    }
  }
</script>

<main class="min-h-screen bg-background">
  <!-- Header -->
  <header class="border-b border-border bg-surface-elevated">
    <div class="container py-6">
      <div class="flex items-center justify-center mb-6">
        <img src="/logo-full.png" alt="LAO Logo" class="h-16 w-auto" />
      </div>
      
      <!-- Quick greet section -->
      <form class="flex gap-3 justify-center max-w-md mx-auto" onsubmit={greet}>
        <Input 
          placeholder="Enter a name..." 
          bind:value={name}
          class="flex-1"
        />
        <Button type="submit" variant="primary">Greet</Button>
      </form>
      {#if greetMsg}
        <p class="text-center mt-3 text-secondary">{greetMsg}</p>
      {/if}
    </div>
  </header>

  <div class="container py-8">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- Prompt-Driven Workflow -->
      <Card title="AI Workflow Generator" subtitle="Describe your workflow in natural language">
        <div class="space-y-4">
          <textarea 
            class="textarea w-full h-24" 
            placeholder="Describe your workflow (e.g. 'Summarize this audio and tag action items')"
            bind:value={prompt}
          ></textarea>
          
          <Button 
            onclick={generateWorkflowFromPrompt} 
            loading={isGenerating}
            disabled={!prompt.trim()}
            variant="primary"
            class="w-full"
          >
            {isGenerating ? 'Generating...' : 'Generate Workflow'}
          </Button>
          
          {#if genError}
            <Alert variant="error" title="Generation Error">
              {genError}
            </Alert>
          {/if}
          
          {#if generatedYaml}
            <div class="space-y-3">
              <h4 class="font-medium text-primary">Generated Workflow YAML</h4>
              <pre class="text-xs bg-surface p-3 rounded-md border overflow-auto">{generatedYaml}</pre>
            </div>
          {/if}
          
          {#if genGraph}
            <div class="space-y-3">
              <h4 class="font-medium text-primary">Workflow Preview</h4>
              <div class="bg-surface p-4 rounded-md border">
                <div class="flex flex-wrap gap-2">
                  {#each genGraph.nodes as node}
                    <div class="px-3 py-2 bg-primary text-white rounded-md text-sm">
                      <div class="font-medium">{node.id}</div>
                      <div class="text-xs opacity-80">{node.run}</div>
                    </div>
                  {/each}
                </div>
                {#if genGraph.edges.length > 0}
                  <div class="mt-3 pt-3 border-t border-border">
                    <div class="text-xs text-muted mb-2">Connections:</div>
                    <div class="flex flex-wrap gap-2">
                      {#each genGraph.edges as edge}
                        <span class="text-xs text-secondary">{edge.from} → {edge.to}</span>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </Card>

      <!-- Workflow Management -->
      <Card title="Workflow Management" subtitle="Load, create, and run workflows">
        <div class="space-y-4">
          <div class="flex gap-2">
            <Button onclick={newWorkflow} variant="secondary" size="small">New</Button>
            <Input 
              placeholder="Workflow YAML path..." 
              bind:value={workflowPath}
              class="flex-1"
            />
            <Button onclick={loadGraph} variant="secondary">Load</Button>
          </div>
          
          <div class="flex gap-2">
            <Button 
              onclick={() => runWorkflow(false)} 
              variant="primary"
              loading={isRunning}
              disabled={!workflowPath || isRunning}
            >
              {isRunning ? 'Running...' : 'Run Sequential'}
            </Button>
            <Button 
              onclick={() => runWorkflow(true)} 
              variant="secondary"
              disabled={!workflowPath || isRunning}
            >
              Run Parallel
            </Button>
          </div>
          
          {#if error}
            <Alert variant="error" title="Workflow Error">
              {error}
            </Alert>
          {/if}
        </div>
      </Card>
    </div>

    <!-- Visual Graph Editor -->
    {#if graph}
      <Card title="Visual Workflow Editor" subtitle="Design your workflow visually" class="mt-8">
        <VisualGraphEditor {graph} on:updateGraph={e => graph = e.detail} on:selectNode={onSelectNode} />
        
        <!-- Node Inspector -->
        {#if selectedNode}
          <div class="mt-6 p-4 bg-surface rounded-md border">
            <h4 class="font-medium text-primary mb-3">Node Inspector</h4>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label class="block text-sm font-medium text-secondary mb-1">Node ID</label>
                <div class="px-3 py-2 bg-background border rounded-md text-sm">{selectedNode.id}</div>
              </div>
              <div>
                <label class="block text-sm font-medium text-secondary mb-1">Plugin Type</label>
                <select bind:value={selectedNode.run} class="select w-full" onchange={(e) => updateSelectedNodeRun(e.target.value)}>
                  {#each plugins as p}
                    <option value={p.name}>{p.name}</option>
                  {/each}
                </select>
              </div>
              <div class="flex items-end">
                <Button onclick={removeSelectedNode} variant="secondary" size="small">Remove Node</Button>
              </div>
            </div>
          </div>
        {/if}
        
        <!-- Add Node Section -->
        <div class="mt-6 p-4 bg-surface rounded-md border">
          <h4 class="font-medium text-primary mb-3">Add New Node</h4>
          <div class="flex gap-3">
            <Input 
              placeholder="Node name (optional)" 
              bind:value={newNodeName}
              class="flex-1"
            />
            <select bind:value={newNodeType} class="select">
              {#if plugins && plugins.length}
                {#each plugins as p}
                  <option value={p.name}>{p.name}</option>
                {/each}
              {:else}
                <option disabled selected>Loading plugins...</option>
              {/if}
            </select>
            <Button onclick={addNode} variant="primary">Add Node</Button>
          </div>
        </div>
        
        <!-- Export/Import Section -->
        <div class="mt-6 p-4 bg-surface rounded-md border">
          <h4 class="font-medium text-primary mb-3">Export/Import Workflow</h4>
          <div class="space-y-4">
            <div class="flex gap-3">
              <Input 
                placeholder="Filename (e.g. new_workflow.yaml)" 
                bind:value={newWorkflowFilename}
                class="flex-1"
              />
              <Button onclick={() => saveWorkflowAsYaml(newWorkflowFilename)} variant="secondary">Save as YAML</Button>
              <Button onclick={exportYAML} variant="secondary">Preview YAML</Button>
            </div>
            
            {#if yamlExport}
              <pre class="text-xs bg-background p-3 rounded-md border overflow-auto">{yamlExport}</pre>
            {/if}
            
            <div class="space-y-2">
              <textarea 
                class="textarea w-full h-32" 
                bind:value={yamlText} 
                placeholder="Paste YAML here to import..."
              ></textarea>
              <Button onclick={importYAML} variant="secondary" size="small">Import YAML</Button>
            </div>
          </div>
        </div>
      </Card>
    {/if}

    <!-- Live Logs -->
    <Card title="Execution Logs" subtitle="Real-time workflow execution status" class="mt-8">
      <div class="bg-slate-900 text-green-400 font-mono text-sm p-4 rounded-md h-64 overflow-y-auto">
        {#if liveLogs.length === 0}
          <div class="text-slate-500">No logs yet. Run a workflow to see execution details.</div>
        {:else}
          {#each liveLogs as log}
            <div class="mb-1">{log}</div>
          {/each}
        {/if}
      </div>
    </Card>
  </div>
</main>

<style>
  .space-y-4 > * + * {
    margin-top: 1rem;
  }
  
  .space-y-3 > * + * {
    margin-top: 0.75rem;
  }
  
  .space-y-2 > * + * {
    margin-top: 0.5rem;
  }
  
  .min-h-screen {
    min-height: 100vh;
  }
  
  .bg-background {
    background-color: var(--color-background);
  }
  
  .bg-surface {
    background-color: var(--color-surface);
  }
  
  .bg-surface-elevated {
    background-color: var(--color-surface-elevated);
  }
  
  .border-border {
    border-color: var(--color-border);
  }
  
  .border-b {
    border-bottom-width: 1px;
  }
  
  .w-full {
    width: 100%;
  }
  
  .h-16 {
    height: 4rem;
  }
  
  .h-24 {
    height: 6rem;
  }
  
  .h-32 {
    height: 8rem;
  }
  
  .h-64 {
    height: 16rem;
  }
  
  .w-auto {
    width: auto;
  }
  
  .max-w-md {
    max-width: 28rem;
  }
  
  .mx-auto {
    margin-left: auto;
    margin-right: auto;
  }
  
  .py-6 {
    padding-top: 1.5rem;
    padding-bottom: 1.5rem;
  }
  
  .py-8 {
    padding-top: 2rem;
    padding-bottom: 2rem;
  }
  
  .text-center {
    text-align: center;
  }
  
  .grid-cols-1 {
    grid-template-columns: repeat(1, minmax(0, 1fr));
  }
  
  .grid-cols-3 {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  
  @media (min-width: 768px) {
    .md\\:grid-cols-3 {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
  
  @media (min-width: 1024px) {
    .lg\\:grid-cols-2 {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  
  .overflow-auto {
    overflow: auto;
  }
  
  .overflow-y-auto {
    overflow-y: auto;
  }
  
  .flex-wrap {
    flex-wrap: wrap;
  }
  
  .bg-red-50 {
    background-color: #fef2f2;
  }
  
  .border-red-200 {
    border-color: #fecaca;
  }
  
  .text-red-700 {
    color: #b91c1c;
  }
  
  .bg-slate-900 {
    background-color: #0f172a;
  }
  
  .text-green-400 {
    color: #4ade80;
  }
  
  .text-slate-500 {
    color: #64748b;
  }
  
  .border-t {
    border-top-width: 1px;
  }
  
  .pt-3 {
    padding-top: 0.75rem;
  }
  
  .rounded-md {
    border-radius: 0.375rem;
  }
  
  .block {
    display: block;
  }
</style>
