/**
 * Conversations store using Svelte 5 Runes.
 *
 * Manages conversation list and selection state with reactive updates.
 * Integrates with Tauri IPC for data fetching.
 */

import type { Conversation, ConversationSummary, ConversationFilters } from "$lib/types";

// Reactive state using Svelte 5 runes
let conversations = $state<ConversationSummary[]>([]);
let selectedId = $state<string | null>(null);
let selectedConversation = $state<Conversation | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

// Active filters
let filters = $state<ConversationFilters>({});

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
 */
export async function select(id: string | null): Promise<void> {
  if (id === selectedId) return;

  selectedId = id;

  if (!id) {
    selectedConversation = null;
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

/**
 * Set conversations directly (for mock data in development).
 */
export function setConversations(data: ConversationSummary[]): void {
  conversations = data;
}

/**
 * Set selected conversation directly (for mock data in development).
 */
export function setSelectedConversation(data: Conversation | null): void {
  selectedConversation = data;
}

/**
 * Set loading state directly (for external control).
 */
export function setLoading(isLoading: boolean): void {
  loading = isLoading;
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
  // Actions
  load,
  select,
  clearSelection,
  setFilters,
  clearFilters,
  setConversations,
  setSelectedConversation,
  setLoading,
};
