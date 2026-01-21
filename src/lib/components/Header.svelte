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
  import { searchStore, filtersStore, conversationsStore, tagsStore, uiStore } from "$lib/stores";

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
