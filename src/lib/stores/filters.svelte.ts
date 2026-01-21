/**
 * Filters store using Svelte 5 Runes.
 *
 * Manages filter state for conversation list with reactive updates.
 * Changes automatically trigger conversation list filtering.
 */

// Reactive filter state using Svelte 5 runes
let projectFilter = $state<string | null>(null);
let dateStart = $state<string | null>(null);
let dateEnd = $state<string | null>(null);
let searchQuery = $state("");

/**
 * Set the project filter.
 */
export function setProject(project: string | null): void {
  projectFilter = project;
}

/**
 * Set the date range filter.
 */
export function setDateRange(start: string | null, end: string | null): void {
  dateStart = start;
  dateEnd = end;
}

/**
 * Set the search query.
 */
export function setSearch(query: string): void {
  searchQuery = query;
}

/**
 * Clear all filters.
 */
export function clearAll(): void {
  projectFilter = null;
  dateStart = null;
  dateEnd = null;
  searchQuery = "";
}

/**
 * Check if any filters are active.
 */
function hasActiveFilters(): boolean {
  return !!(projectFilter || dateStart || dateEnd || searchQuery);
}

/**
 * Get the current filter state as a ConversationFilters object.
 */
function toConversationFilters() {
  return {
    project: projectFilter ?? undefined,
    dateStart: dateStart ?? undefined,
    dateEnd: dateEnd ?? undefined,
  };
}

// Export reactive getters
export const filtersStore = {
  get projectFilter() {
    return projectFilter;
  },
  get dateStart() {
    return dateStart;
  },
  get dateEnd() {
    return dateEnd;
  },
  get searchQuery() {
    return searchQuery;
  },
  get hasActiveFilters() {
    return hasActiveFilters();
  },
  get asConversationFilters() {
    return toConversationFilters();
  },
  // Actions
  setProject,
  setDateRange,
  setSearch,
  clearAll,
};
