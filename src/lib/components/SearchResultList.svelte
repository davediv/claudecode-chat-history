<script lang="ts">
  /**
   * Search results list component.
   *
   * Features:
   * - Displays search results with highlighted terms
   * - Shows result count: "12 results for 'query'"
   * - Each result shows: snippet, project name, date
   * - Click to open conversation and scroll to match
   * - Empty state when no results found
   */
  import type { SearchResult, ConversationSummary } from "$lib/types";
  import { formatRelativeDate } from "$lib/utils";
  import { SvelteMap } from "svelte/reactivity";

  interface Props {
    /** Search results to display */
    results: SearchResult[];
    /** Current search query (for display) */
    query: string;
    /** Conversation summaries for metadata lookup */
    conversations: ConversationSummary[];
    /** Currently selected conversation ID */
    selectedId?: string | null;
    /** Handler for result selection */
    onSelect?: (conversationId: string) => void;
    /** Whether search is in progress */
    isSearching?: boolean;
  }

  let {
    results,
    query,
    conversations,
    selectedId = null,
    onSelect,
    isSearching = false,
  }: Props = $props();

  // Create a map for quick conversation lookup
  const conversationMap = $derived.by(() => {
    const map = new SvelteMap<string, ConversationSummary>();
    for (const conv of conversations) {
      map.set(conv.id, conv);
    }
    return map;
  });

  /**
   * Get conversation metadata by ID.
   */
  function getConversation(id: string): ConversationSummary | undefined {
    return conversationMap.get(id);
  }

  function handleClick(conversationId: string) {
    onSelect?.(conversationId);
  }

  function handleKeydown(event: KeyboardEvent, conversationId: string) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect?.(conversationId);
    }
  }
</script>

<div class="search-results" role="listbox" aria-label="Search results">
  <!-- Screen reader announcement for result count -->
  {#if query && !isSearching}
    <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
      {#if results.length === 0}
        No results found for "{query}"
      {:else if results.length === 1}
        1 result found for "{query}"
      {:else}
        {results.length} results found for "{query}"
      {/if}
    </div>
  {/if}

  <!-- Results header -->
  {#if query && !isSearching}
    <div class="results-header">
      <span class="results-count">
        {#if results.length === 0}
          No results for "{query}"
        {:else if results.length === 1}
          1 result for "{query}"
        {:else}
          {results.length} results for "{query}"
        {/if}
      </span>
    </div>
  {/if}

  <!-- Loading state -->
  {#if isSearching}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <span>Searching...</span>
    </div>
  {:else if results.length === 0 && query}
    <!-- Empty state -->
    <div class="empty-state">
      <svg
        class="empty-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <circle cx="11" cy="11" r="8"></circle>
        <path d="m21 21-4.35-4.35"></path>
        <path d="M8 8l6 6M14 8l-6 6"></path>
      </svg>
      <p class="empty-title">No results found</p>
      <p class="empty-description">Try different keywords or check your spelling.</p>
    </div>
  {:else}
    <!-- Results list -->
    <div class="results-list">
      {#each results as result (result.conversationId)}
        {@const conv = getConversation(result.conversationId)}
        <div
          class="result-item"
          class:selected={selectedId === result.conversationId}
          role="option"
          aria-selected={selectedId === result.conversationId}
          tabindex="0"
          onclick={() => handleClick(result.conversationId)}
          onkeydown={(e) => handleKeydown(e, result.conversationId)}
        >
          <div class="result-header">
            <span class="project-name" title={conv?.projectName}>
              {conv?.projectName ?? "Unknown project"}
            </span>
            <span class="timestamp">
              {conv ? formatRelativeDate(conv.lastTime) : ""}
            </span>
          </div>
          <p class="snippet">
            {#if result.snippet}
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html result.snippet}
            {:else}
              <span class="no-snippet">No preview available</span>
            {/if}
          </p>
          <div class="result-footer">
            {#if result.matchCount > 1}
              <span class="match-count">
                {result.matchCount} matches
              </span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* Screen reader only - visually hidden but accessible */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .search-results {
    height: 100%;
    width: 100%;
    overflow-y: auto;
    background-color: var(--color-bg-secondary);
  }

  .results-header {
    position: sticky;
    top: 0;
    z-index: 1;
    padding: 0.75rem 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .results-count {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .loading-spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: calc(100% - 3rem);
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
  }

  .empty-icon {
    width: 3rem;
    height: 3rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  .empty-title {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .empty-description {
    margin: 0;
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .results-list {
    display: flex;
    flex-direction: column;
  }

  .result-item {
    padding: 0.75rem 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background-color 0.15s ease;
    outline: none;
  }

  .result-item:hover {
    background-color: var(--color-bg-tertiary);
  }

  .result-item.selected {
    background-color: var(--color-bg-tertiary);
    border-left: 3px solid var(--color-accent);
    padding-left: calc(1rem - 3px);
  }

  .result-item:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.375rem;
    gap: 0.5rem;
  }

  .project-name {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .timestamp {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .snippet {
    margin: 0 0 0.375rem 0;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    line-height: 1.5;
    /* Two-line clamp fallback */
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* Highlight matched terms in snippet */
  .snippet :global(mark) {
    background-color: rgba(250, 204, 21, 0.3);
    color: inherit;
    padding: 0.125rem 0;
    border-radius: 2px;
  }

  .no-snippet {
    font-style: italic;
    color: var(--color-text-muted);
  }

  .result-footer {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .match-count {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    background-color: var(--color-bg-tertiary);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
  }
</style>
