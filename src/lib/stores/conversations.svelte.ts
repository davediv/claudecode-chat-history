/**
 * Conversations store using Svelte 5 Runes.
 *
 * Manages conversation list and selection state with reactive updates.
 * Integrates with Tauri IPC for data fetching.
 * Implements LRU caching for conversation details.
 * Persists selected conversation ID for session restoration.
 */

import type { Conversation, ConversationSummary, ConversationFilters } from "$lib/types";
import { SvelteMap } from "svelte/reactivity";
import { getStorageItem, setStorageItem, removeStorageItem, STORAGE_KEYS } from "$lib/utils";

// Reactive state using Svelte 5 runes
let conversations = $state<ConversationSummary[]>([]);
let selectedId = $state<string | null>(null);
let selectedConversation = $state<Conversation | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

// Active filters
let filters = $state<ConversationFilters>({});

// LRU cache for conversation details
const CACHE_MAX_SIZE = 100; // Maximum number of cached conversations (configurable)
const conversationCache = new SvelteMap<string, Conversation>();
const cacheAccessOrder: string[] = []; // Track access order for LRU

/**
 * Get a conversation from cache if available.
 */
function getCached(id: string): Conversation | undefined {
  const cached = conversationCache.get(id);
  if (cached) {
    // Move to end of access order (most recently used)
    const index = cacheAccessOrder.indexOf(id);
    if (index > -1) {
      cacheAccessOrder.splice(index, 1);
    }
    cacheAccessOrder.push(id);
  }
  return cached;
}

/**
 * Add a conversation to cache, evicting oldest if at capacity.
 */
function addToCache(id: string, conversation: Conversation): void {
  // If at capacity, remove least recently used
  while (conversationCache.size >= CACHE_MAX_SIZE && cacheAccessOrder.length > 0) {
    const oldest = cacheAccessOrder.shift();
    if (oldest) {
      conversationCache.delete(oldest);
    }
  }

  // Add to cache
  conversationCache.set(id, conversation);
  cacheAccessOrder.push(id);
}

/**
 * Clear the conversation cache.
 */
export function clearCache(): void {
  conversationCache.clear();
  cacheAccessOrder.length = 0;
}

/**
 * Get cache statistics.
 */
export function getCacheStats(): { size: number; maxSize: number } {
  return { size: conversationCache.size, maxSize: CACHE_MAX_SIZE };
}

/**
 * Get filtered conversations based on active filters.
 */
function getFilteredConversations(): ConversationSummary[] {
  let filtered = conversations;

  if (filters.project) {
    filtered = filtered.filter((c) => c.projectName === filters.project);
  }

  if (filters.dateStart) {
    filtered = filtered.filter((c) => c.lastTime >= filters.dateStart!);
  }

  if (filters.dateEnd) {
    filtered = filtered.filter((c) => c.lastTime <= filters.dateEnd!);
  }

  return filtered;
}

/**
 * Load conversations from backend.
 * Uses Tauri IPC when available, falls back to mock data in browser.
 */
export async function load(options?: ConversationFilters): Promise<void> {
  loading = true;
  error = null;

  try {
    // Update filters if provided
    if (options) {
      filters = { ...filters, ...options };
    }

    // Try to use Tauri IPC if available
    if (typeof window !== "undefined" && "__TAURI__" in window) {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<ConversationSummary[]>("get_conversations", {
        filters: Object.keys(filters).length > 0 ? filters : null,
      });
      conversations = result;
    } else {
      // Development mode: data comes from +page.svelte mock generator
      // Keep existing conversations if they were set externally
      console.log("[conversations store] Running in browser mode, using mock data");
    }
  } catch (err) {
    console.error("Failed to load conversations:", err);
    error = err instanceof Error ? err.message : "Failed to load conversations";
  } finally {
    loading = false;
  }
}

/**
 * Select a conversation by ID and load its full details.
 * Uses LRU cache to avoid refetching previously viewed conversations.
 * Persists selection to localStorage for session restoration.
 */
export async function select(id: string | null): Promise<void> {
  if (id === selectedId) return;

  selectedId = id;

  // Persist selection to localStorage
  if (id) {
    setStorageItem(STORAGE_KEYS.SELECTED_CONVERSATION, id);
  } else {
    removeStorageItem(STORAGE_KEYS.SELECTED_CONVERSATION);
  }

  if (!id) {
    selectedConversation = null;
    return;
  }

  // Check cache first
  const cached = getCached(id);
  if (cached) {
    selectedConversation = cached;
    return;
  }

  loading = true;
  error = null;

  try {
    // Try to use Tauri IPC if available
    if (typeof window !== "undefined" && "__TAURI__" in window) {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<Conversation>("get_conversation", { id });
      selectedConversation = result;
      // Add to cache for future access
      addToCache(id, result);
    } else {
      // Development mode: conversation details come from +page.svelte
      // Keep existing selectedConversation if set externally
      console.log("[conversations store] Running in browser mode, selection handled externally");
    }
  } catch (err) {
    console.error("Failed to load conversation:", err);
    error = err instanceof Error ? err.message : "Failed to load conversation";
    selectedConversation = null;
  } finally {
    loading = false;
  }
}

/**
 * Clear the current selection.
 */
export function clearSelection(): void {
  selectedId = null;
  selectedConversation = null;
  removeStorageItem(STORAGE_KEYS.SELECTED_CONVERSATION);
}

/**
 * Restore the previously selected conversation from localStorage.
 * Should be called on app startup after conversations are loaded.
 */
export async function restoreSelection(): Promise<void> {
  const savedId = getStorageItem<string>(STORAGE_KEYS.SELECTED_CONVERSATION);
  if (savedId && conversations.some((c) => c.id === savedId)) {
    await select(savedId);
  }
}

/**
 * Get the previously selected conversation ID from localStorage.
 * Useful for checking if restoration is needed without triggering load.
 */
export function getPersistedSelectionId(): string | null {
  return getStorageItem<string>(STORAGE_KEYS.SELECTED_CONVERSATION);
}

/**
 * Update filters and optionally reload.
 */
export function setFilters(newFilters: ConversationFilters, reload = true): void {
  filters = { ...newFilters };
  if (reload) {
    load();
  }
}

/**
 * Clear all filters.
 */
export function clearFilters(): void {
  filters = {};
  load();
}

// Track if a reload is in progress to prevent concurrent reloads
let reloadInProgress = false;

/**
 * Reload conversations in response to file watcher updates.
 * Preserves current selection and scroll position.
 * Optionally invalidates cache for updated conversations.
 * Prevents concurrent reloads via locking.
 */
export async function reload(): Promise<void> {
  // Prevent concurrent reloads
  if (reloadInProgress) {
    console.log("[conversations store] Reload already in progress, skipping");
    return;
  }

  reloadInProgress = true;

  try {
    // Store current selection to preserve it
    const currentSelectedId = selectedId;

    // Reload the conversation list
    await load();

    // If we had a selection, try to restore it
    if (currentSelectedId) {
      // Check if the conversation still exists
      const stillExists = conversations.some((c) => c.id === currentSelectedId);
      if (stillExists) {
        // Invalidate cache for this conversation so we get fresh data
        conversationCache.delete(currentSelectedId);
        const accessIndex = cacheAccessOrder.indexOf(currentSelectedId);
        if (accessIndex > -1) {
          cacheAccessOrder.splice(accessIndex, 1);
        }
        // Re-select to reload fresh data
        selectedId = null; // Reset to force re-fetch
        await select(currentSelectedId);
      } else {
        // Conversation was deleted, clear selection
        clearSelection();
      }
    }
  } finally {
    reloadInProgress = false;
  }
}

/**
 * Set conversations directly (for mock data in development).
 */
export function setConversations(data: ConversationSummary[]): void {
  conversations = data;
}

/**
 * Set selected conversation directly (for mock data in development).
 * Also adds to cache for consistent caching behavior.
 */
export function setSelectedConversation(data: Conversation | null): void {
  selectedConversation = data;
  // Add to cache if valid conversation
  if (data && data.id) {
    addToCache(data.id, data);
  }
}

/**
 * Set loading state directly (for external control).
 */
export function setLoading(isLoading: boolean): void {
  loading = isLoading;
}

/**
 * Toggle bookmark status for a conversation.
 * Updates both the conversations list and selected conversation if applicable.
 */
export async function toggleBookmark(conversationId: string): Promise<boolean> {
  // Try to use Tauri IPC if available
  if (typeof window !== "undefined" && "__TAURI__" in window) {
    const { invoke } = await import("@tauri-apps/api/core");
    const newStatus = await invoke<boolean>("toggle_bookmark", { conversationId });

    // Update the conversation in the list
    conversations = conversations.map((c) =>
      c.id === conversationId ? { ...c, bookmarked: newStatus } : c
    );

    // Update cache if this conversation is cached
    const cached = conversationCache.get(conversationId);
    if (cached) {
      conversationCache.set(conversationId, { ...cached, bookmarked: newStatus });
    }

    // Update selected conversation if it's the one being toggled
    if (selectedConversation && selectedConversation.id === conversationId) {
      selectedConversation = { ...selectedConversation, bookmarked: newStatus };
    }

    return newStatus;
  } else {
    // Development mode: toggle locally
    const conv = conversations.find((c) => c.id === conversationId);
    const newStatus = conv ? !conv.bookmarked : true;

    conversations = conversations.map((c) =>
      c.id === conversationId ? { ...c, bookmarked: newStatus } : c
    );

    if (selectedConversation && selectedConversation.id === conversationId) {
      selectedConversation = { ...selectedConversation, bookmarked: newStatus };
    }

    return newStatus;
  }
}

// Export reactive getters
export const conversationsStore = {
  get conversations() {
    return conversations;
  },
  get filteredConversations() {
    return getFilteredConversations();
  },
  get selectedId() {
    return selectedId;
  },
  get selectedConversation() {
    return selectedConversation;
  },
  get loading() {
    return loading;
  },
  get error() {
    return error;
  },
  get filters() {
    return filters;
  },
  get cacheStats() {
    return getCacheStats();
  },
  // Actions
  load,
  reload,
  select,
  clearSelection,
  restoreSelection,
  getPersistedSelectionId,
  setFilters,
  clearFilters,
  setConversations,
  setSelectedConversation,
  setLoading,
  clearCache,
  toggleBookmark,
};
