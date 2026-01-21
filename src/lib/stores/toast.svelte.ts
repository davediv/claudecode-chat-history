/**
 * Toast notification store using Svelte 5 Runes.
 *
 * Manages a stack of toast notifications with auto-dismiss functionality.
 */

export type ToastType = "success" | "error" | "info" | "warning";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration: number;
}

interface ToastOptions {
  message: string;
  type?: ToastType;
  duration?: number;
}

// Generate unique ID for each toast
let toastIdCounter = 0;
function generateId(): string {
  return `toast-${++toastIdCounter}-${Date.now()}`;
}

// Reactive toast array using Svelte 5 runes
let toasts = $state<Toast[]>([]);

/**
 * Add a new toast notification.
 * Returns the toast ID for manual dismissal if needed.
 */
export function addToast(options: ToastOptions): string {
  const id = generateId();
  const toast: Toast = {
    id,
    message: options.message,
    type: options.type ?? "info",
    duration: options.duration ?? 3000,
  };

  toasts = [...toasts, toast];

  // Auto-dismiss after duration
  if (toast.duration > 0) {
    setTimeout(() => {
      dismissToast(id);
    }, toast.duration);
  }

  return id;
}

/**
 * Dismiss a specific toast by ID.
 */
export function dismissToast(id: string): void {
  toasts = toasts.filter((t) => t.id !== id);
}

/**
 * Dismiss all toasts.
 */
export function clearAllToasts(): void {
  toasts = [];
}

/**
 * Get the current list of toasts (reactive).
 */
export function getToasts(): Toast[] {
  return toasts;
}

// Convenience functions for common toast types
export const toast = {
  success: (message: string, duration?: number) => addToast({ message, type: "success", duration }),
  error: (message: string, duration?: number) => addToast({ message, type: "error", duration }),
  info: (message: string, duration?: number) => addToast({ message, type: "info", duration }),
  warning: (message: string, duration?: number) => addToast({ message, type: "warning", duration }),
};
