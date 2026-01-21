/**
 * Local storage utilities for session persistence.
 *
 * Provides type-safe localStorage access with fallbacks for SSR and errors.
 */

const STORAGE_PREFIX = "claude-history:";

/**
 * Safely get a value from localStorage.
 * Returns null if not found, in SSR, or on error.
 */
export function getStorageItem<T>(key: string): T | null {
  if (typeof window === "undefined") return null;

  try {
    const item = localStorage.getItem(STORAGE_PREFIX + key);
    if (item === null) return null;
    return JSON.parse(item) as T;
  } catch {
    return null;
  }
}

/**
 * Safely set a value in localStorage.
 * Silently fails in SSR or on error.
 */
export function setStorageItem<T>(key: string, value: T): void {
  if (typeof window === "undefined") return;

  try {
    localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(value));
  } catch {
    // Silently fail (storage full, disabled, etc.)
  }
}

/**
 * Remove a value from localStorage.
 */
export function removeStorageItem(key: string): void {
  if (typeof window === "undefined") return;

  try {
    localStorage.removeItem(STORAGE_PREFIX + key);
  } catch {
    // Silently fail
  }
}

// Storage keys
export const STORAGE_KEYS = {
  SELECTED_CONVERSATION: "selectedConversation",
  FILTERS: "filters",
} as const;
