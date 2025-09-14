<script>
  export let title = '';
  export let subtitle = '';
  export let description = '';
  export let padding = 'normal';

  $: classes = [
    'card',
    padding === 'small' ? 'p-4' : padding === 'large' ? 'p-8' : 'p-6'
  ].join(' ');
</script>

<div class={classes}>
  {#if title || subtitle}
    <header class="card-header mb-4">
      {#if title}
        <h2 class="text-xl font-semibold text-primary">{title}</h2>
      {/if}
      {#if subtitle}
        <p class="text-sm text-secondary mt-1">{subtitle}</p>
      {/if}
    </header>
  {/if}
  
  {#if description}
    <p class="text-muted mb-4">{description}</p>
  {/if}
  
  <div class="card-content">
    <slot />
  </div>
</div>

<style>
  .card-header {
    border-bottom: 1px solid var(--color-border);
    padding-bottom: var(--space-4);
    margin-bottom: var(--space-4);
  }
  
  .card-header:last-child {
    border-bottom: none;
    padding-bottom: 0;
    margin-bottom: 0;
  }
</style>