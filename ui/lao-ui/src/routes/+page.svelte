<script>
  import { invoke } from "@tauri-apps/api/core";
  import { mkdir, writeTextFile } from "@tauri-apps/plugin-fs";

  let name = $state("");
  let greetMsg = $state("");

  // FlowBuilder state
  let workflowPath = $state("");
  let graph = $state(null);
  let error = $state("");
  let newNodeName = $state("");
  let newNodeType = $state("Echo");
  let yamlExport = $state("");

  // Prompt-driven workflow state
  let prompt = $state("");
  let generatedYaml = $state("");
  let genError = $state("");
  let genGraph = $state(null);

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
  }

  async function generateWorkflowFromPrompt() {
    genError = "";
    generatedYaml = "";
    genGraph = null;
    if (!prompt.trim()) {
      genError = "Please enter a prompt.";
      return;
    }
    // Add a note to the user
    genError = "Generating workflow... this may take a few seconds.";
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
    }
  }
</script>

<main class="container">
  <img src="/logo-full.png" alt="LAO Logo" class="lao-logo" />
  <form class="row" onsubmit={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>

  <section style="margin-top: 2em;">
    <h2>Prompt-Driven Workflow</h2>
    <input placeholder="Describe your workflow (e.g. 'Summarize this audio and tag action items')" bind:value={prompt} style="width: 60%;" />
    <button onclick={generateWorkflowFromPrompt}>Generate Workflow</button>
    {#if genError}
      <p style="color: red;">{genError}</p>
    {/if}
    {#if generatedYaml}
      <h3>Generated Workflow YAML</h3>
      <pre>{generatedYaml}</pre>
    {/if}
    {#if genGraph}
      <h3>Visualized Workflow</h3>
      <div class="graph-vis">
        <div class="nodes">
          {#each genGraph.nodes as node}
            <div class="node-card">
              <b>{node.id}</b>: {node.run} (<i>{node.status}</i>)
            </div>
          {/each}
        </div>
        <div class="edges">
          <h4>Edges</h4>
          <ul>
            {#each genGraph.edges as edge}
              <li>{edge.from} → {edge.to}</li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  </section>

  <section style="margin-top: 2em;">
    <h2>Visual Flow Builder</h2>
    <input placeholder="Workflow YAML path..." bind:value={workflowPath} />
    <button onclick={loadGraph}>Load Workflow</button>
    {#if error}
      <p style="color: red;">{error}</p>
    {/if}
    {#if graph}
      <div style="margin-top: 1em;">
        <h3>Nodes</h3>
        <ul>
          {#each graph.nodes as node}
            <li>
              <b>{node.id}</b>: {node.run} (<i>{node.status}</i>)
              <button onclick={() => removeNode(node.id)}>Remove</button>
            </li>
          {/each}
        </ul>
        <h3>Add Node</h3>
        <input placeholder="Node name (optional)" bind:value={newNodeName} />
        <select bind:value={newNodeType}>
          <option>Echo</option>
          <option>Whisper</option>
          <option>Ollama</option>
        </select>
        <button onclick={addNode}>Add Node</button>
        <h3>Edges</h3>
        <ul>
          {#each graph.edges as edge}
            <li>{edge.from} → {edge.to}</li>
          {/each}
        </ul>
        <button onclick={exportYAML}>Export as YAML</button>
        {#if yamlExport}
          <pre>{yamlExport}</pre>
        {/if}
      </div>
    {/if}
  </section>
</main>

<style>
.lao-logo {
  display: block;
  margin: 0 auto 2em auto;
  max-width: 300px;
  height: auto;
}
.graph-vis {
  display: flex;
  flex-direction: row;
  gap: 2em;
  margin-top: 1em;
  justify-content: center;
}
.nodes {
  display: flex;
  flex-direction: column;
  gap: 1em;
}
.node-card {
  background: #222;
  color: #fff;
  border-radius: 8px;
  padding: 1em 2em;
  box-shadow: 0 2px 8px rgba(0,0,0,0.2);
  min-width: 180px;
  text-align: left;
}
.edges ul {
  list-style: none;
  padding: 0;
}
.edges li {
  color: #aaa;
  font-size: 1em;
}

.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte-kit:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
