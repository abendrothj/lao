<script>
  export let value = '';
  export let options = [];
  export let label = '';
  export let placeholder = 'Select an option';
  export let error = '';
  export let required = false;
  export let disabled = false;

  let id = `select-${Math.random().toString(36).substr(2, 9)}`;

  $: selectClasses = [
    'select',
    error ? 'select-error' : ''
  ].filter(Boolean).join(' ');
</script>

<div class="select-group">
  {#if label}
    <label for={id} class="select-label">
      {label}
      {#if required}
        <span class="text-error">*</span>
      {/if}
    </label>
  {/if}
  
  <select
    {id}
    {required}
    {disabled}
    class={selectClasses}
    bind:value
    {...$$restProps}
  >
    {#if placeholder}
      <option value="" disabled selected={!value}>{placeholder}</option>
    {/if}
    {#each options as option}
      {#if typeof option === 'string'}
        <option value={option}>{option}</option>
      {:else}
        <option value={option.value}>{option.label}</option>
      {/if}
    {/each}
  </select>
  
  {#if error}
    <p class="select-error-text">{error}</p>
  {/if}
</div>

<style>
  .select-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  
  .select-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-primary);
  }
  
  .select-error {
    border-color: var(--color-error);
  }
  
  .select-error:focus {
    border-color: var(--color-error);
    box-shadow: 0 0 0 3px rgb(239 68 68 / 0.1);
  }
  
  .select-error-text {
    font-size: var(--text-xs);
    color: var(--color-error);
    margin: 0;
  }
  
  .text-error {
    color: var(--color-error);
  }
</style>