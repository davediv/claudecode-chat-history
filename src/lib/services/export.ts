/**
 * Export service for converting conversations to Markdown and saving to files.
 *
 * Uses Tauri's dialog and fs plugins to provide native save dialogs
 * and file system access.
 */

import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { Conversation, Message, ContentBlock } from "$lib/types";

/**
 * Format a date for display in exported Markdown.
 */
function formatDate(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toLocaleString("en-US", {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  } catch {
    return isoString;
  }
}

/**
 * Format a date for use in filenames (YYYY-MM-DD format).
 */
function formatDateForFilename(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toISOString().split("T")[0];
  } catch {
    return "unknown-date";
  }
}

/**
 * Sanitize a string for use in filenames.
 * Removes/replaces characters that are invalid in filenames.
 */
function sanitizeFilename(name: string): string {
  const sanitized = name
    .replace(/[<>:"/\\|?*]/g, "-") // Replace invalid chars
    .replace(/\s+/g, "-") // Replace whitespace with dashes
    .replace(/-+/g, "-") // Collapse multiple dashes
    .replace(/^-|-$/g, "") // Trim leading/trailing dashes
    .toLowerCase();

  return sanitized || "conversation"; // Fallback if empty
}

/**
 * Convert a content block to Markdown.
 */
function contentBlockToMarkdown(block: ContentBlock): string {
  switch (block.type) {
    case "text":
      return block.content;

    case "code": {
      const lang = block.language || "";
      return `\`\`\`${lang}\n${block.content}\n\`\`\``;
    }

    case "tool_use":
      return `> **Tool Use: ${block.toolName || "Unknown"}**\n>\n> \`\`\`\n> ${block.content.split("\n").join("\n> ")}\n> \`\`\``;

    case "tool_result":
      return `> **Tool Result${block.toolName ? `: ${block.toolName}` : ""}**\n>\n> \`\`\`\n> ${block.content.split("\n").join("\n> ")}\n> \`\`\``;

    default:
      return block.content;
  }
}

/**
 * Convert a message to Markdown.
 */
function messageToMarkdown(message: Message): string {
  const roleLabel = message.role.charAt(0).toUpperCase() + message.role.slice(1);
  const timestamp = formatDate(message.timestamp);

  const contentParts = message.content.map(contentBlockToMarkdown);
  const content = contentParts.join("\n\n");

  return `### ${roleLabel}\n\n*${timestamp}*\n\n${content}`;
}

/**
 * Generate Markdown content from a conversation.
 */
export function generateMarkdown(conversation: Conversation): string {
  const lines: string[] = [];

  // Header
  lines.push(`# ${conversation.projectName}`);
  lines.push("");
  lines.push(`**Started:** ${formatDate(conversation.startTime)}`);
  lines.push(`**Last Updated:** ${formatDate(conversation.lastTime)}`);
  lines.push(`**Messages:** ${conversation.messages.length}`);

  // Tags if present
  if (conversation.tags && conversation.tags.length > 0) {
    lines.push(`**Tags:** ${conversation.tags.join(", ")}`);
  }

  lines.push("");
  lines.push("---");
  lines.push("");

  // Messages
  if (conversation.messages.length === 0) {
    lines.push("*This conversation contains no messages.*");
    lines.push("");
  } else {
    for (const message of conversation.messages) {
      lines.push(messageToMarkdown(message));
      lines.push("");
      lines.push("---");
      lines.push("");
    }
  }

  return lines.join("\n");
}

/**
 * Generate a default filename for the export.
 * Limits filename length to stay within filesystem limits.
 */
export function generateFilename(conversation: Conversation): string {
  const projectPart = sanitizeFilename(conversation.projectName);
  const datePart = formatDateForFilename(conversation.lastTime);

  // Limit project part to reasonable length
  // Date is 10 chars, underscore is 1, .md is 3 = 14 chars overhead
  // Most filesystems allow 255 bytes, use 200 as safe limit
  const maxProjectLength = 200;
  const truncatedProject = projectPart.slice(0, maxProjectLength);

  return `${truncatedProject}_${datePart}.md`;
}

/**
 * Export a conversation to a Markdown file.
 * Opens a save dialog and writes the file to the selected location.
 *
 * @returns true if export was successful, false if cancelled or failed
 */
export async function exportConversation(conversation: Conversation): Promise<boolean> {
  try {
    const markdown = generateMarkdown(conversation);
    const defaultFilename = generateFilename(conversation);

    // Show save dialog
    const filePath = await save({
      defaultPath: defaultFilename,
      filters: [
        {
          name: "Markdown",
          extensions: ["md"],
        },
        {
          name: "All Files",
          extensions: ["*"],
        },
      ],
      title: "Export Conversation",
    });

    // User cancelled
    if (!filePath) {
      return false;
    }

    // Write the file
    await writeTextFile(filePath, markdown);

    return true;
  } catch (error) {
    console.error("Failed to export conversation:", error);
    return false;
  }
}
