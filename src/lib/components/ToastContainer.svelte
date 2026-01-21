<script lang="ts">
  /**
   * Toast container component.
   *
   * Renders all active toasts in a stack at the bottom-right corner.
   * Should be placed at the root of the application.
   */
  import Toast from "./Toast.svelte";
  import { getToasts, dismissToast } from "$lib/stores/toast.svelte";

  // Get reactive toasts list
  const toasts = $derived(getToasts());
</script>

<div class="toast-container" aria-label="Notifications">
  {#each toasts as toast (toast.id)}
    <Toast id={toast.id} message={toast.message} type={toast.type} onDismiss={dismissToast} />
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column-reverse;
    gap: 0.5rem;
    z-index: 1000;
    pointer-events: none;
  }

  .toast-container :global(.toast) {
    pointer-events: auto;
  }

  /* Responsive: move to full width on narrow screens */
  @media (max-width: 480px) {
    .toast-container {
      left: 1rem;
      right: 1rem;
    }

    .toast-container :global(.toast) {
      max-width: none;
      width: 100%;
    }
  }
</style>
