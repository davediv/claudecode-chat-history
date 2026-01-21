/**
 * UI store using Svelte 5 Runes.
 *
 * Manages transient UI state like loading indicators, sidebar visibility,
 * and re-exports toast functionality for convenience.
 */

import { addToast, dismissToast, clearAllToasts, toast } from "./toast.svelte";

// Reactive UI state using Svelte 5 runes
let isLoading = $state(false);
let sidebarCollapsed = $state(false);
let analyticsModalOpen = $state(false);

/**
 * Set global loading state.
 */
export function setLoading(loading: boolean): void {
  isLoading = loading;
}

/**
 * Toggle sidebar collapsed state.
 */
export function toggleSidebar(): void {
  sidebarCollapsed = !sidebarCollapsed;
}

/**
 * Set sidebar collapsed state directly.
 */
export function setSidebarCollapsed(collapsed: boolean): void {
  sidebarCollapsed = collapsed;
}

/**
 * Open the analytics modal.
 */
export function openAnalyticsModal(): void {
  analyticsModalOpen = true;
}

/**
 * Close the analytics modal.
 */
export function closeAnalyticsModal(): void {
  analyticsModalOpen = false;
}

// Re-export toast functions for convenience
export const showToast = addToast;
export { dismissToast, clearAllToasts, toast };

// Export reactive getters
export const uiStore = {
  get isLoading() {
    return isLoading;
  },
  get sidebarCollapsed() {
    return sidebarCollapsed;
  },
  get analyticsModalOpen() {
    return analyticsModalOpen;
  },
  // Actions
  setLoading,
  toggleSidebar,
  setSidebarCollapsed,
  openAnalyticsModal,
  closeAnalyticsModal,
  showToast,
  dismissToast,
  clearAllToasts,
  toast,
};
