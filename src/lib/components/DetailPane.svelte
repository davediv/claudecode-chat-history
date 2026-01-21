<script lang="ts">
  /**
   * Detail pane component for displaying conversation content.
   *
   * Features:
   * - Full conversation view with messages
   * - Empty state when no conversation selected
   * - Loading state while fetching conversation
   */
  import type { Snippet } from "svelte";

  interface Props {
    /** Whether a conversation is selected */
    hasSelection?: boolean;
    /** Whether conversation is loading */
    isLoading?: boolean;
    /** Content to render when a conversation is selected */
    children?: Snippet;
  }

  let { hasSelection = false, isLoading = false, children }: Props = $props();
</script>

<main class="detail-pane" aria-label="Conversation details" aria-busy={isLoading}>
  {#if isLoading}
    <div class="detail-loading" role="status" aria-live="polite">
      <div class="loading-spinner" aria-hidden="true"></div>
      <span>Loading conversation...</span>
    </div>
  {:else if !hasSelection}
    <div class="detail-empty">
      <svg
        class="empty-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path
          d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 0 1-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
        ></path>
      </svg>
      <p>Select a conversation</p>
      <p class="empty-hint">Choose a conversation from the sidebar to view its contents</p>
      <p class="keyboard-hint">
        <kbd>j</kbd>/<kbd>k</kbd> to navigate, <kbd>Enter</kbd> to select
      </p>
    </div>
  {:else}
    <div class="detail-content">
      {#if children}
        {@render children()}
      {/if}
    </div>
  {/if}
</main>

<style>
  .detail-pane {
    flex: 1;
    height: 100%;
    background-color: var(--color-bg-primary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0; /* Allow flex shrinking */
  }

  .detail-loading,
  .detail-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
  }

  .loading-spinner {
    width: 2rem;
    height: 2rem;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-icon {
    width: 4rem;
    height: 4rem;
    margin-bottom: 1rem;
    opacity: 0.4;
  }

  .detail-empty p {
    margin: 0;
    font-size: 1.125rem;
  }

  .empty-hint {
    font-size: 0.875rem;
    margin-top: 0.5rem !important;
    opacity: 0.7;
  }

  .keyboard-hint {
    font-size: 0.75rem;
    margin-top: 1rem !important;
    opacity: 0.5;
  }

  .keyboard-hint kbd {
    display: inline-block;
    padding: 0.125rem 0.375rem;
    margin: 0 0.125rem;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-family: "SF Mono", Monaco, Menlo, Consolas, monospace;
    font-size: 0.6875rem;
  }

  .detail-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  /* Responsive: full width on narrow screens */
  @media (max-width: 640px) {
    .detail-pane {
      width: 100%;
    }
  }
</style>
