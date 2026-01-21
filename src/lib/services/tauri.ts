/**
 * Tauri IPC service wrapper.
 *
 * Provides type-safe wrappers around Tauri invoke calls
 * with proper error handling and browser fallback.
 */

import type {
  Conversation,
  ConversationSummary,
  ConversationFilters,
  ProjectInfo,
  SearchResult,
  ConversationsUpdatedEvent,
  TagInfo,
} from "$lib/types";

/**
 * Error types for Tauri IPC operations.
 */
export class TauriError extends Error {
  constructor(
    message: string,
    public readonly code: string
  ) {
    super(message);
    this.name = "TauriError";
  }
}

export class NotFoundError extends TauriError {
  constructor(message: string) {
    super(message, "NOT_FOUND");
    this.name = "NotFoundError";
  }
}

export class NetworkError extends TauriError {
  constructor(message: string) {
    super(message, "NETWORK_ERROR");
    this.name = "NetworkError";
  }
}

/**
 * Check if running in Tauri environment.
 */
export function isTauriAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

/**
 * Get the Tauri invoke function.
 * Returns null if not in Tauri environment.
 */
async function getInvoke(): Promise<typeof import("@tauri-apps/api/core").invoke | null> {
  if (!isTauriAvailable()) {
    return null;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke;
}

/**
 * Wrap Tauri errors in typed error classes.
 */
function wrapError(error: unknown, operation: string): TauriError {
  if (error instanceof TauriError) {
    return error;
  }

  const message = error instanceof Error ? error.message : String(error);

  if (message.includes("not found") || message.includes("NotFound")) {
    return new NotFoundError(`${operation}: ${message}`);
  }

  return new TauriError(`${operation} failed: ${message}`, "UNKNOWN");
}

/**
 * Get list of conversations with optional filters.
 *
 * @param filters - Optional filters for project, date range
 * @returns Array of conversation summaries sorted by lastTime desc
 * @throws TauriError if operation fails
 */
export async function getConversations(
  filters?: ConversationFilters
): Promise<ConversationSummary[]> {
  const invoke = await getInvoke();

  if (!invoke) {
    console.log("[tauri service] Not in Tauri environment, returning empty array");
    return [];
  }

  try {
    const result = await invoke<ConversationSummary[]>("get_conversations", {
      filters: filters && Object.keys(filters).length > 0 ? filters : null,
    });
    return result;
  } catch (error) {
    throw wrapError(error, "getConversations");
  }
}

/**
 * Get full conversation details by ID.
 *
 * @param id - Conversation ID
 * @returns Full conversation with all messages
 * @throws NotFoundError if conversation not found
 * @throws TauriError if operation fails
 */
export async function getConversation(id: string): Promise<Conversation> {
  const invoke = await getInvoke();

  if (!invoke) {
    throw new TauriError("Not in Tauri environment", "NOT_AVAILABLE");
  }

  try {
    const result = await invoke<Conversation>("get_conversation", { id });
    return result;
  } catch (error) {
    throw wrapError(error, "getConversation");
  }
}

/**
 * Get list of all projects with conversation counts.
 *
 * @returns Array of project info sorted alphabetically by name
 * @throws TauriError if operation fails
 */
export async function getProjects(): Promise<ProjectInfo[]> {
  const invoke = await getInvoke();

  if (!invoke) {
    console.log("[tauri service] Not in Tauri environment, returning empty array");
    return [];
  }

  try {
    const result = await invoke<ProjectInfo[]>("get_projects");
    return result;
  } catch (error) {
    throw wrapError(error, "getProjects");
  }
}

/**
 * Search conversations by query with optional filters.
 *
 * @param query - Search query string (min 2 characters)
 * @param filters - Optional filters for project, date range
 * @returns Array of search results with snippets and match counts
 * @throws TauriError if operation fails
 */
export async function searchConversations(
  query: string,
  filters?: ConversationFilters
): Promise<SearchResult[]> {
  const invoke = await getInvoke();

  if (!invoke) {
    console.log("[tauri service] Not in Tauri environment, returning empty array");
    return [];
  }

  // Enforce minimum query length
  if (query.length < 2) {
    return [];
  }

  try {
    const result = await invoke<SearchResult[]>("search_conversations", {
      query,
      filters: filters && Object.keys(filters).length > 0 ? filters : null,
    });
    return result;
  } catch (error) {
    throw wrapError(error, "searchConversations");
  }
}

/**
 * Unlisten function type from Tauri events API.
 */
export type UnlistenFn = () => void;

/**
 * Event name for conversations updated events from backend.
 */
export const CONVERSATIONS_UPDATED_EVENT = "conversations-updated";

/**
 * Listen for conversations-updated events from the backend file watcher.
 * Returns an unlisten function to clean up the listener.
 *
 * @param callback - Function to call when conversations are updated
 * @returns Promise resolving to unlisten function, or null if not in Tauri
 */
export async function listenToConversationsUpdated(
  callback: (event: ConversationsUpdatedEvent) => void
): Promise<UnlistenFn | null> {
  if (!isTauriAvailable()) {
    console.log("[tauri service] Not in Tauri environment, skipping event listener");
    return null;
  }

  try {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<ConversationsUpdatedEvent>(
      CONVERSATIONS_UPDATED_EVENT,
      (event) => {
        console.log("[tauri service] Received conversations-updated event:", event.payload);
        callback(event.payload);
      }
    );
    console.log("[tauri service] Listening for conversations-updated events");
    return unlisten;
  } catch (error) {
    console.error("[tauri service] Failed to listen for conversations-updated:", error);
    return null;
  }
}

/**
 * Toggle the bookmark status of a conversation.
 *
 * @param conversationId - ID of the conversation to toggle
 * @returns The new bookmark status (true if now bookmarked, false if unbookmarked)
 * @throws TauriError if operation fails
 */
export async function toggleBookmark(conversationId: string): Promise<boolean> {
  const invoke = await getInvoke();

  if (!invoke) {
    throw new TauriError("Not in Tauri environment", "NOT_AVAILABLE");
  }

  try {
    const result = await invoke<boolean>("toggle_bookmark", { conversationId });
    return result;
  } catch (error) {
    throw wrapError(error, "toggleBookmark");
  }
}

/**
 * Set tags for a conversation.
 * Replaces all existing tags with the provided list.
 *
 * @param conversationId - ID of the conversation
 * @param tags - Array of tag strings (will be normalized: lowercase, trimmed, deduplicated)
 * @returns The normalized, sorted array of tags that were set
 * @throws TauriError if operation fails
 */
export async function setTags(conversationId: string, tags: string[]): Promise<string[]> {
  const invoke = await getInvoke();

  if (!invoke) {
    throw new TauriError("Not in Tauri environment", "NOT_AVAILABLE");
  }

  try {
    const result = await invoke<string[]>("set_tags", { conversationId, tags });
    return result;
  } catch (error) {
    throw wrapError(error, "setTags");
  }
}

/**
 * Get all unique tags across all conversations with usage counts.
 *
 * @returns Array of tag info sorted alphabetically by tag name
 * @throws TauriError if operation fails
 */
export async function getAllTags(): Promise<TagInfo[]> {
  const invoke = await getInvoke();

  if (!invoke) {
    console.log("[tauri service] Not in Tauri environment, returning empty array");
    return [];
  }

  try {
    const result = await invoke<TagInfo[]>("get_all_tags");
    return result;
  } catch (error) {
    throw wrapError(error, "getAllTags");
  }
}

/**
 * Tauri service object for convenience import.
 */
export const tauriService = {
  isTauriAvailable,
  getConversations,
  getConversation,
  getProjects,
  searchConversations,
  toggleBookmark,
  setTags,
  getAllTags,
  listenToConversationsUpdated,
};
