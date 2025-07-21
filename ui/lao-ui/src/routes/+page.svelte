<script>
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  // FlowBuilder state
  let workflowPath = $state("");
  let graph = $state(null);
  let error = $state("");
  let newNodeName = $state("");
  let newNodeType = $state("Echo");
  let yamlExport = $state("");

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
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://kit.svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <form class="row" onsubmit={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>

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
