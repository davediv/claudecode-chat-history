<script lang="ts">
  /**
   * Error Boundary component for graceful error handling.
   *
   * Features:
   * - Catches errors from child components
   * - Displays friendly error message with retry option
   * - Logs errors to console for debugging
   * - Maintains app stability when components fail
   */
  import { onMount } from "svelte";

  interface Props {
    /** Title shown in error state */
    title?: string;
    /** Description shown in error state */
    description?: string;
    /** Children to render */
    children: import("svelte").Snippet;
  }

  let {
    title = "Something went wrong",
    description = "An error occurred while displaying this content.",
    children,
  }: Props = $props();

  let hasError = $state(false);
  let errorMessage = $state<string | null>(null);

  // Reset error state
  function retry() {
    hasError = false;
    errorMessage = null;
  }

  // Capture errors from window error events
  onMount(() => {
    function handleError(event: ErrorEvent) {
      console.error("[ErrorBoundary] Uncaught error:", event.error);
      hasError = true;
      errorMessage = event.message || "Unknown error";
      event.preventDefault();
    }

    function handleUnhandledRejection(event: PromiseRejectionEvent) {
      console.error("[ErrorBoundary] Unhandled promise rejection:", event.reason);
      hasError = true;
      errorMessage = event.reason?.message || "Unhandled promise rejection";
      event.preventDefault();
    }

    window.addEventListener("error", handleError);
    window.addEventListener("unhandledrejection", handleUnhandledRejection);

    return () => {
      window.removeEventListener("error", handleError);
      window.removeEventListener("unhandledrejection", handleUnhandledRejection);
    };
  });
</script>

{#if hasError}
  <div class="error-boundary" role="alert" aria-live="assertive">
    <div class="error-content">
      <svg
        class="error-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="12" y1="8" x2="12" y2="12"></line>
        <line x1="12" y1="16" x2="12.01" y2="16"></line>
      </svg>

      <h2 class="error-title">{title}</h2>
      <p class="error-description">{description}</p>

      {#if errorMessage}
        <details class="error-details">
          <summary>Error details</summary>
          <code>{errorMessage}</code>
        </details>
      {/if}

      <button type="button" class="retry-button" onclick={retry}>
        <svg
          class="retry-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          aria-hidden="true"
        >
          <polyline points="23 4 23 10 17 10"></polyline>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
        Try again
      </button>
    </div>
  </div>
{:else}
  {@render children()}
{/if}

<style>
  .error-boundary {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 200px;
    padding: 2rem;
    background-color: var(--color-bg-primary);
  }

  .error-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    max-width: 400px;
    text-align: center;
  }

  .error-icon {
    width: 3rem;
    height: 3rem;
    margin-bottom: 1rem;
    color: var(--color-error, #ef4444);
  }

  .error-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .error-description {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .error-details {
    width: 100%;
    margin-bottom: 1rem;
    text-align: left;
  }

  .error-details summary {
    cursor: pointer;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    padding: 0.5rem;
  }

  .error-details summary:hover {
    color: var(--color-text-secondary);
  }

  .error-details code {
    display: block;
    padding: 0.75rem;
    margin-top: 0.5rem;
    font-family: monospace;
    font-size: 0.75rem;
    background-color: var(--color-bg-tertiary);
    border-radius: 6px;
    color: var(--color-error, #ef4444);
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .retry-button {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: white;
    background-color: var(--color-accent);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      transform 0.1s ease;
  }

  .retry-button:hover {
    background-color: var(--color-accent-hover, #2563eb);
  }

  .retry-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .retry-button:active {
    transform: scale(0.98);
  }

  .retry-icon {
    width: 1rem;
    height: 1rem;
  }
</style>
