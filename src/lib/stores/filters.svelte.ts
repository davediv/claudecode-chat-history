/**
 * Filters store using Svelte 5 Runes.
 *
 * Manages filter state for conversation list with reactive updates.
 * Changes automatically trigger conversation list filtering.
 * Persists filter state to localStorage for session restoration.
 */

import { getStorageItem, setStorageItem, STORAGE_KEYS } from "$lib/utils";

// Type for persisted filter state
interface PersistedFilters {
  project: string | null;
  dateStart: string | null;
  dateEnd: string | null;
}

// Load persisted filters from localStorage
function loadPersistedFilters(): PersistedFilters {
  const stored = getStorageItem<PersistedFilters>(STORAGE_KEYS.FILTERS);
  return stored ?? { project: null, dateStart: null, dateEnd: null };
}

// Initialize with persisted values
const initialFilters = loadPersistedFilters();

// Reactive filter state using Svelte 5 runes
let projectFilter = $state<string | null>(initialFilters.project);
let dateStart = $state<string | null>(initialFilters.dateStart);
let dateEnd = $state<string | null>(initialFilters.dateEnd);
let searchQuery = $state("");

/**
 * Persist current filter state to localStorage.
 */
function persistFilters(): void {
  setStorageItem<PersistedFilters>(STORAGE_KEYS.FILTERS, {
    project: projectFilter,
    dateStart: dateStart,
    dateEnd: dateEnd,
  });
}

/**
 * Set the project filter.
 */
export function setProject(project: string | null): void {
  projectFilter = project;
  persistFilters();
}

/**
 * Set the date range filter.
 */
export function setDateRange(start: string | null, end: string | null): void {
  dateStart = start;
  dateEnd = end;
  persistFilters();
}

/**
 * Set the search query.
 * Note: Search query is NOT persisted (intentionally fresh each session).
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
  persistFilters();
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
