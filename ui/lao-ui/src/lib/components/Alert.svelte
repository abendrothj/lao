<script>
  export let variant = 'info'; // 'info', 'success', 'warning', 'error'
  export let title = '';
  export let dismissible = false;
  export let onDismiss = () => {};

  $: alertClasses = [
    'alert',
    `alert-${variant}`,
    dismissible ? 'alert-dismissible' : ''
  ].filter(Boolean).join(' ');

  $: iconPath = {
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
    success: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
    warning: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.728-.833-2.498 0L4.316 16.5c-.77.833.192 2.5 1.732 2.5z',
    error: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z'
  }[variant];
</script>

<div class={alertClasses} role="alert">
  <div class="alert-content">
    <div class="alert-icon">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={iconPath}></path>
      </svg>
    </div>
    
    <div class="alert-text">
      {#if title}
        <div class="alert-title">{title}</div>
      {/if}
      <div class="alert-message">
        <slot />
      </div>
    </div>
    
    {#if dismissible}
      <button class="alert-dismiss" onclick={onDismiss} aria-label="Dismiss">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    {/if}
  </div>
</div>

<style>
  .alert {
    border-radius: var(--radius-md);
    padding: var(--space-4);
    border: 1px solid;
  }
  
  .alert-content {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
  }
  
  .alert-icon {
    flex-shrink: 0;
    margin-top: 1px;
  }
  
  .alert-text {
    flex: 1;
    min-width: 0;
  }
  
  .alert-title {
    font-weight: var(--font-medium);
    margin-bottom: var(--space-1);
  }
  
  .alert-message {
    font-size: var(--text-sm);
  }
  
  .alert-dismiss {
    flex-shrink: 0;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    transition: background-color var(--transition-fast);
  }
  
  .alert-dismiss:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }
  
  /* Variant styles */
  .alert-info {
    background-color: #eff6ff;
    border-color: #bfdbfe;
    color: #1d4ed8;
  }
  
  .alert-success {
    background-color: #f0fdf4;
    border-color: #bbf7d0;
    color: #166534;
  }
  
  .alert-warning {
    background-color: #fffbeb;
    border-color: #fed7aa;
    color: #d97706;
  }
  
  .alert-error {
    background-color: #fef2f2;
    border-color: #fecaca;
    color: #dc2626;
  }
  
  /* Dark theme adjustments */
  @media (prefers-color-scheme: dark) {
    .alert-info {
      background-color: #1e3a8a;
      border-color: #3b82f6;
      color: #bfdbfe;
    }
    
    .alert-success {
      background-color: #166534;
      border-color: #10b981;
      color: #bbf7d0;
    }
    
    .alert-warning {
      background-color: #d97706;
      border-color: #f59e0b;
      color: #fed7aa;
    }
    
    .alert-error {
      background-color: #dc2626;
      border-color: #ef4444;
      color: #fecaca;
    }
    
    .alert-dismiss:hover {
      background-color: rgba(255, 255, 255, 0.1);
    }
  }
  
  .w-4 { width: 1rem; }
  .h-4 { height: 1rem; }
  .w-5 { width: 1.25rem; }
  .h-5 { height: 1.25rem; }
</style>