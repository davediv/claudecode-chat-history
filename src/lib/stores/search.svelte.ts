/**
 * Search store using Svelte 5 Runes.
 *
 * Manages search state with debounced search trigger
 * and integration with Tauri IPC for full-text search.
 */

import type { SearchResult, ConversationFilters } from "$lib/types";
import { searchConversations, isTauriAvailable } from "$lib/services";

// Reactive state using Svelte 5 runes
let query = $state("");
let results = $state<SearchResult[]>([]);
let isSearching = $state(false);
let error = $state<string | null>(null);

// Active search filters
let activeFilters = $state<ConversationFilters>({});

// Debounce timeout
let debounceTimeout: ReturnType<typeof setTimeout> | null = null;

// Default debounce delay in milliseconds
const DEBOUNCE_MS = 300;

/**
 * Execute search with current query and filters.
 */
async function executeSearch(searchQuery: string): Promise<void> {
  // Clear results for empty query
  if (!searchQuery.trim()) {
    results = [];
    error = null;
    return;
  }

  // Skip search for very short queries
  if (searchQuery.length < 2) {
    return;
  }

  isSearching = true;
  error = null;

  try {
    if (isTauriAvailable()) {
      const searchResults = await searchConversations(
        searchQuery,
        Object.keys(activeFilters).length > 0 ? activeFilters : undefined
      );
      results = searchResults;
    } else {
      // Development mode: use mock search
      console.log("[search store] Running in browser mode, using mock search");
      results = [];
    }
  } catch (err) {
    console.error("Search failed:", err);
    error = err instanceof Error ? err.message : "Search failed";
    results = [];
  } finally {
    isSearching = false;
  }
}

/**
 * Trigger a search with debouncing.
 *
 * @param searchQuery - Query string to search for
 * @param debounceMs - Debounce delay in milliseconds (default 300ms)
 */
export function search(searchQuery: string, debounceMs: number = DEBOUNCE_MS): void {
  query = searchQuery;

  // Clear any pending debounce
  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
  }

  // Clear results immediately for empty query
  if (!searchQuery.trim()) {
    results = [];
    error = null;
    return;
  }

  // Debounce the actual search
  debounceTimeout = setTimeout(() => {
    executeSearch(searchQuery);
    debounceTimeout = null;
  }, debounceMs);
}

/**
 * Execute search immediately without debouncing.
 *
 * @param searchQuery - Query string to search for
 */
export async function searchImmediate(searchQuery: string): Promise<void> {
  // Clear any pending debounce
  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
    debounceTimeout = null;
  }

  query = searchQuery;
  await executeSearch(searchQuery);
}

/**
 * Clear the search and reset state.
 */
export function clearSearch(): void {
  // Clear any pending debounce
  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
    debounceTimeout = null;
  }

  query = "";
  results = [];
  error = null;
  isSearching = false;
}

/**
 * Set search filters.
 *
 * @param filters - Filters to apply to search
 * @param rerun - Whether to re-run the current search with new filters
 */
export function setFilters(filters: ConversationFilters, rerun = true): void {
  activeFilters = { ...filters };
  if (rerun && query.trim()) {
    executeSearch(query);
  }
}

/**
 * Clear search filters.
 *
 * @param rerun - Whether to re-run the current search without filters
 */
export function clearFilters(rerun = true): void {
  activeFilters = {};
  if (rerun && query.trim()) {
    executeSearch(query);
  }
}

/**
 * Set results directly (for mock data in development).
 */
export function setResults(data: SearchResult[]): void {
  results = data;
}

/**
 * Check if search mode is active (has a query).
 */
export function isActive(): boolean {
  return query.trim().length > 0;
}

// Export reactive getters
export const searchStore = {
  get query() {
    return query;
  },
  get results() {
    return results;
  },
  get isSearching() {
    return isSearching;
  },
  get error() {
    return error;
  },
  get filters() {
    return activeFilters;
  },
  get isActive() {
    return query.trim().length > 0;
  },
  // Actions
  search,
  searchImmediate,
  clearSearch,
  setFilters,
  clearFilters,
  setResults,
};
