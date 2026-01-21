<script lang="ts">
  /**
   * Toast notification component.
   *
   * Features:
   * - Configurable message, type (success, error, info, warning), and duration
   * - Slide-in animation from right
   * - Dismiss button for manual dismissal
   * - Accessible with role="alert" and aria-live
   */
  import type { ToastType } from "$lib/stores/toast.svelte";

  interface Props {
    /** Unique identifier for the toast */
    id: string;
    /** Message to display */
    message: string;
    /** Toast type for styling */
    type?: ToastType;
    /** Handler for dismissing the toast */
    onDismiss?: (id: string) => void;
  }

  let { id, message, type = "info", onDismiss }: Props = $props();

  function handleDismiss() {
    onDismiss?.(id);
  }

  // Icon paths for each type
  const icons: Record<ToastType, string> = {
    success: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
    error: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
    warning:
      "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
    info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
  };
</script>

<div class="toast toast-{type}" role="alert" aria-live="polite" aria-atomic="true">
  <div class="toast-icon">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d={icons[type]}></path>
    </svg>
  </div>
  <p class="toast-message">{message}</p>
  <button class="toast-dismiss" aria-label="Dismiss notification" onclick={handleDismiss}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M6 18L18 6M6 6l12 12"></path>
    </svg>
  </button>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 280px;
    max-width: 400px;
    padding: 0.875rem 1rem;
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(100%);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .toast-icon {
    flex-shrink: 0;
    width: 1.25rem;
    height: 1.25rem;
  }

  .toast-icon svg {
    width: 100%;
    height: 100%;
  }

  .toast-message {
    flex: 1;
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    line-height: 1.4;
  }

  .toast-dismiss {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-muted);
    transition:
      color 0.15s ease,
      background-color 0.15s ease;
  }

  .toast-dismiss:hover {
    color: var(--color-text-primary);
    background-color: var(--color-bg-tertiary);
  }

  .toast-dismiss svg {
    width: 1rem;
    height: 1rem;
  }

  /* Type-specific styling */
  .toast-success {
    border-left: 3px solid var(--color-success);
  }

  .toast-success .toast-icon {
    color: var(--color-success);
  }

  .toast-error {
    border-left: 3px solid var(--color-error);
  }

  .toast-error .toast-icon {
    color: var(--color-error);
  }

  .toast-warning {
    border-left: 3px solid var(--color-warning);
  }

  .toast-warning .toast-icon {
    color: var(--color-warning);
  }

  .toast-info {
    border-left: 3px solid var(--color-info);
  }

  .toast-info .toast-icon {
    color: var(--color-info);
  }

  /* Respect reduced motion preference - toast appears instantly */
  @media (prefers-reduced-motion: reduce) {
    .toast {
      animation: none;
    }

    .toast-dismiss {
      transition: none;
    }
  }
</style>
