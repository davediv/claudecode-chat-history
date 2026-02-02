<script lang="ts">
  /**
   * Header component with search input and filter controls.
   *
   * Features:
   * - Search input with debouncing
   * - Project filter dropdown
   * - Date range filter
   * - Tag filter dropdown
   * - Filter integration with stores
   */
  import SearchInput from "./SearchInput.svelte";
  import ProjectFilter from "./ProjectFilter.svelte";
  import DateRangePicker from "./DateRangePicker.svelte";
  import BookmarkedFilter from "./BookmarkedFilter.svelte";
  import TagFilter from "./TagFilter.svelte";
  import {
    searchStore,
    filtersStore,
    conversationsStore,
    tagsStore,
    uiStore,
    groupedViewStore,
  } from "$lib/stores";

  interface Props {
    /** Handler for search changes */
    onSearch?: (query: string) => void;
    /** Handler for filter changes */
    onFilterChange?: () => void;
  }

  let { onSearch, onFilterChange }: Props = $props();

  /**
   * Handle search query changes.
   */
  function handleSearch(query: string) {
    searchStore.search(query);
    onSearch?.(query);
  }

  /**
   * Handle project filter changes.
   */
  function handleProjectChange() {
    // Reload conversations with new filter
    conversationsStore.load(filtersStore.asConversationFilters);
    onFilterChange?.();
  }

  /**
   * Handle date filter changes.
   */
  function handleDateChange() {
    // Reload conversations with new filter
    conversationsStore.load(filtersStore.asConversationFilters);
    onFilterChange?.();
  }

  /**
   * Handle bookmarked filter changes.
   */
  function handleBookmarkedChange() {
    // Reload conversations with new filter
    conversationsStore.load(filtersStore.asConversationFilters);
    onFilterChange?.();
  }

  /**
   * Handle tag filter changes.
   */
  function handleTagsChange(tags: string[]) {
    filtersStore.setTags(tags);
    // Reload conversations with new filter
    conversationsStore.load(filtersStore.asConversationFilters);
    onFilterChange?.();
  }

  /**
   * Open the analytics modal.
   */
  function handleOpenAnalytics() {
    uiStore.openAnalyticsModal();
  }

  /**
   * Toggle between grouped and flat view modes.
   */
  function handleToggleGroupedView() {
    groupedViewStore.toggleGroupedView();
  }

  /**
   * Expand all project groups.
   */
  function handleExpandAll() {
    const projectNames = Array.from(
      new Set(conversationsStore.conversations.map((c) => c.projectName))
    );
    groupedViewStore.expandAll(projectNames);
  }

  /**
   * Collapse all project groups.
   */
  function handleCollapseAll() {
    groupedViewStore.collapseAll();
  }
</script>

<header class="header">
  <div class="header-left">
    <h1 class="app-title">Claude Code History</h1>
  </div>

  <div class="header-center">
    <SearchInput
      value={searchStore.query}
      isSearching={searchStore.isSearching}
      onSearch={handleSearch}
    />
  </div>

  <div class="header-right">
    <div class="filters">
      <ProjectFilter onChange={handleProjectChange} />
      <DateRangePicker onChange={handleDateChange} />
      <BookmarkedFilter onChange={handleBookmarkedChange} />
      <TagFilter
        allTags={tagsStore.allTags}
        selectedTags={filtersStore.tagsFilter}
        onTagsChange={handleTagsChange}
      />
      <button
        type="button"
        class="analytics-button"
        onclick={handleOpenAnalytics}
        aria-label="View usage analytics"
        title="Usage Analytics"
      >
        <svg
          class="analytics-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M18 20V10"></path>
          <path d="M12 20V4"></path>
          <path d="M6 20v-6"></path>
        </svg>
      </button>

      <!-- View mode toggle -->
      <div class="view-mode-controls">
        <button
          type="button"
          class="view-toggle-button"
          class:active={groupedViewStore.groupedViewEnabled}
          onclick={handleToggleGroupedView}
          aria-label={groupedViewStore.groupedViewEnabled
            ? "Switch to flat view"
            : "Switch to grouped view"}
          title={groupedViewStore.groupedViewEnabled ? "Flat view" : "Grouped view"}
        >
          {#if groupedViewStore.groupedViewEnabled}
            <!-- Grouped icon (folder tree) -->
            <svg
              class="view-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
              ></path>
              <line x1="12" y1="11" x2="12" y2="17"></line>
              <line x1="9" y1="14" x2="15" y2="14"></line>
            </svg>
          {:else}
            <!-- Flat icon (list) -->
            <svg
              class="view-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="8" y1="6" x2="21" y2="6"></line>
              <line x1="8" y1="12" x2="21" y2="12"></line>
              <line x1="8" y1="18" x2="21" y2="18"></line>
              <line x1="3" y1="6" x2="3.01" y2="6"></line>
              <line x1="3" y1="12" x2="3.01" y2="12"></line>
              <line x1="3" y1="18" x2="3.01" y2="18"></line>
            </svg>
          {/if}
        </button>

        {#if groupedViewStore.groupedViewEnabled}
          <button
            type="button"
            class="expand-collapse-button"
            onclick={handleExpandAll}
            aria-label="Expand all projects"
            title="Expand all"
          >
            <svg
              class="view-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="15 3 21 3 21 9"></polyline>
              <polyline points="9 21 3 21 3 15"></polyline>
              <line x1="21" y1="3" x2="14" y2="10"></line>
              <line x1="3" y1="21" x2="10" y2="14"></line>
            </svg>
          </button>
          <button
            type="button"
            class="expand-collapse-button"
            onclick={handleCollapseAll}
            aria-label="Collapse all projects"
            title="Collapse all"
          >
            <svg
              class="view-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="4 14 10 14 10 20"></polyline>
              <polyline points="20 10 14 10 14 4"></polyline>
              <line x1="14" y1="10" x2="21" y2="3"></line>
              <line x1="3" y1="21" x2="10" y2="14"></line>
            </svg>
          </button>
        {/if}
      </div>
    </div>
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

  .filters {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .analytics-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease;
  }

  .analytics-button:hover {
    background: var(--color-bg-tertiary);
  }

  .analytics-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .analytics-icon {
    width: 1rem;
    height: 1rem;
  }

  .view-mode-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: 0.5rem;
    padding-left: 0.5rem;
    border-left: 1px solid var(--color-border);
  }

  .view-toggle-button,
  .expand-collapse-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease;
  }

  .view-toggle-button:hover,
  .expand-collapse-button:hover {
    background: var(--color-bg-tertiary);
  }

  .view-toggle-button:focus-visible,
  .expand-collapse-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .view-toggle-button.active {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .view-icon {
    width: 1rem;
    height: 1rem;
  }

  /* Responsive adjustments */
  @media (max-width: 900px) {
    .filters {
      flex-direction: column;
      gap: 0.25rem;
    }
  }

  @media (max-width: 640px) {
    .app-title {
      display: none;
    }

    .header-center {
      max-width: none;
    }

    .header-right {
      display: none;
    }
  }
</style>
