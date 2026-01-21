/**
 * Clipboard service using Tauri clipboard API.
 *
 * Provides clipboard operations for the desktop application
 * with proper error handling and fallback behavior.
 */
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

/**
 * Copy text to the system clipboard.
 *
 * @param text - The text to copy to clipboard
 * @returns true if copy succeeded, false otherwise
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await writeText(text);
    return true;
  } catch (error) {
    console.error("Failed to copy to clipboard:", error);
    return false;
  }
}
