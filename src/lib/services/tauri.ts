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
 * Tauri service object for convenience import.
 */
export const tauriService = {
  isTauriAvailable,
  getConversations,
  getConversation,
  getProjects,
  searchConversations,
};
