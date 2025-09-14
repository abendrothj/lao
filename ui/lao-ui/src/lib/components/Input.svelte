<script>
  export let type = 'text';
  export let placeholder = '';
  export let value = '';
  export let label = '';
  export let error = '';
  export let required = false;
  export let disabled = false;

  let id = `input-${Math.random().toString(36).substr(2, 9)}`;

  $: inputClasses = [
    'input',
    error ? 'input-error' : ''
  ].filter(Boolean).join(' ');
</script>

<div class="input-group">
  {#if label}
    <label for={id} class="input-label">
      {label}
      {#if required}
        <span class="text-error">*</span>
      {/if}
    </label>
  {/if}
  
  <input
    {id}
    {type}
    {placeholder}
    {required}
    {disabled}
    class={inputClasses}
    bind:value
    {...$$restProps}
  />
  
  {#if error}
    <p class="input-error-text">{error}</p>
  {/if}
</div>

<style>
  .input-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  
  .input-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-primary);
  }
  
  .input-error {
    border-color: var(--color-error);
  }
  
  .input-error:focus {
    border-color: var(--color-error);
    box-shadow: 0 0 0 3px rgb(239 68 68 / 0.1);
  }
  
  .input-error-text {
    font-size: var(--text-xs);
    color: var(--color-error);
    margin: 0;
  }
  
  .text-error {
    color: var(--color-error);
  }
</style>