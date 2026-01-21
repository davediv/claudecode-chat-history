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
  import { searchStore, filtersStore, conversationsStore, tagsStore } from "$lib/stores";

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
