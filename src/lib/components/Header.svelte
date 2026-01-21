<script lang="ts">
  /**
   * Header component with search input and filter controls.
   *
   * Features:
   * - Search input for filtering conversations
   * - Project filter dropdown (placeholder)
   * - Date range filter (placeholder)
   */

  interface Props {
    /** Current search query */
    searchQuery?: string;
    /** Handler for search query changes */
    onSearch?: (query: string) => void;
  }

  let { searchQuery = $bindable(""), onSearch }: Props = $props();

  function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;
    onSearch?.(searchQuery);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      searchQuery = "";
      onSearch?.("");
    }
  }
</script>

<header class="header">
  <div class="header-left">
    <h1 class="app-title">Claude Code History</h1>
  </div>

  <div class="header-center">
    <div class="search-container">
      <svg
        class="search-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="11" cy="11" r="8"></circle>
        <path d="m21 21-4.35-4.35"></path>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search conversations..."
        value={searchQuery}
        oninput={handleSearch}
        onkeydown={handleKeydown}
      />
      {#if searchQuery}
        <button
          class="clear-button"
          aria-label="Clear search"
          onclick={() => {
            searchQuery = "";
            onSearch?.("");
          }}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18M6 6l12 12"></path>
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <div class="header-right">
    <!-- Placeholder for filter controls -->
    <button class="filter-button">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
      </svg>
      Filters
    </button>
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    height: var(--header-height);
    padding: 0 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .header-left {
    flex-shrink: 0;
  }

  .header-center {
    flex: 1;
    max-width: 400px;
    margin: 0 auto;
  }

  .header-right {
    flex-shrink: 0;
  }

  .app-title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
  }

  .search-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 0.75rem;
    width: 1rem;
    height: 1rem;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 2rem 0.5rem 2.25rem;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    transition: border-color 0.15s ease;
  }

  .search-input::placeholder {
    color: var(--color-text-muted);
  }

  .search-input:focus {
    border-color: var(--color-accent);
  }

  .clear-button {
    position: absolute;
    right: 0.5rem;
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
      background-color 0.15s ease,
      color 0.15s ease;
  }

  .clear-button:hover {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .clear-button svg {
    width: 0.875rem;
    height: 0.875rem;
  }

  .filter-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    transition:
      border-color 0.15s ease,
      background-color 0.15s ease;
  }

  .filter-button:hover {
    background-color: var(--color-border-light);
    border-color: var(--color-accent);
  }

  .filter-button svg {
    width: 1rem;
    height: 1rem;
  }

  /* Responsive adjustments */
  @media (max-width: 640px) {
    .app-title {
      display: none;
    }

    .header-center {
      max-width: none;
    }
  }
</style>
