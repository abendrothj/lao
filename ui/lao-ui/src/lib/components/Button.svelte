<script>
  export let variant = 'primary';
  export let size = 'medium';
  export let disabled = false;
  export let loading = false;
  export let type = 'button';
  export let onclick = undefined;

  $: classes = [
    'btn',
    variant === 'primary' ? 'btn-primary' : 'btn-secondary',
    size === 'small' ? 'btn-sm' : size === 'large' ? 'btn-lg' : '',
    loading ? 'btn-loading' : ''
  ].filter(Boolean).join(' ');
</script>

<button 
  {type}
  class={classes}
  {disabled}
  onclick={onclick}
  {...$$restProps}
>
  {#if loading}
    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
  {/if}
  <slot />
</button>

<style>
  .btn-sm {
    padding: var(--space-2) var(--space-3);
    font-size: var(--text-xs);
  }
  
  .btn-lg {
    padding: var(--space-4) var(--space-6);
    font-size: var(--text-lg);
  }
  
  .btn-loading {
    cursor: wait;
  }
  
  .w-4 { width: 1rem; }
  .h-4 { height: 1rem; }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .animate-spin {
    animation: spin 1s linear infinite;
  }
</style>