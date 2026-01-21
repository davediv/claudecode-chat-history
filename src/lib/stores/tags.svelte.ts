/**
 * Tags store using Svelte 5 Runes.
 *
 * Manages the list of all available tags across conversations
 * for autocomplete and filtering functionality.
 */

import type { TagInfo } from "$lib/types";
import { getAllTags } from "$lib/services/tauri";

// Reactive state for all tags
let allTags = $state<TagInfo[]>([]);
let isLoading = $state(false);
let lastLoadError = $state<string | null>(null);

/**
 * Load all tags from the backend.
 */
async function load(): Promise<void> {
  if (isLoading) return;

  isLoading = true;
  lastLoadError = null;

  try {
    const tags = await getAllTags();
    allTags = tags;
  } catch (error) {
    lastLoadError = error instanceof Error ? error.message : String(error);
    console.error("[tags store] Failed to load tags:", error);
  } finally {
    isLoading = false;
  }
}

/**
 * Refresh tags (alias for load).
 */
async function refresh(): Promise<void> {
  return load();
}

/**
 * Clear all tags from the store.
 */
function clear(): void {
  allTags = [];
  lastLoadError = null;
}

// Export reactive getters and actions
export const tagsStore = {
  get allTags() {
    return allTags;
  },
  get isLoading() {
    return isLoading;
  },
  get lastLoadError() {
    return lastLoadError;
  },
  // Actions
  load,
  refresh,
  clear,
};
