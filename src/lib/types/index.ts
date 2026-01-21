// TypeScript interfaces and types for Claude Code Chat History Viewer

/**
 * Token count for input/output tracking
 */
export interface TokenCount {
  input: number;
  output: number;
}

/**
 * A content block within a message.
 * Messages can contain multiple blocks of different types.
 */
export interface ContentBlock {
  type: "text" | "code" | "tool_use" | "tool_result";
  content: string;
  /** Programming language for code blocks */
  language?: string;
  /** Tool name for tool_use/tool_result blocks */
  toolName?: string;
}

/**
 * A single message in a conversation.
 */
export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: ContentBlock[];
  timestamp: string; // ISO 8601 format
  tokenCount?: TokenCount;
}

/**
 * A complete conversation with all messages.
 * Used when viewing conversation details.
 */
export interface Conversation {
  /** Unique ID derived from file path + session */
  id: string;
  /** Original project directory path */
  projectPath: string;
  /** Display name (last path segments) */
  projectName: string;
  /** First message timestamp */
  startTime: string; // ISO 8601 format
  /** Last message timestamp */
  lastTime: string; // ISO 8601 format
  /** All messages in the conversation */
  messages: Message[];
  /** Total token usage */
  totalTokens: TokenCount;
  /** User bookmark status (MVP extension point) */
  bookmarked?: boolean;
  /** User-defined tags (MVP extension point) */
  tags?: string[];
}

/**
 * Lightweight conversation summary for list view.
 * Does not include full message content for performance.
 */
export interface ConversationSummary {
  id: string;
  projectName: string;
  startTime: string; // ISO 8601 format
  lastTime: string; // ISO 8601 format
  /** First user message, truncated to 100 characters */
  preview: string;
  /** Total number of messages */
  messageCount: number;
  /** Whether this conversation is bookmarked */
  bookmarked: boolean;
}

/**
 * Filter options for querying conversations.
 */
export interface ConversationFilters {
  /** Filter by project name */
  project?: string;
  /** Start of date range (inclusive) */
  dateStart?: string; // ISO 8601 format
  /** End of date range (inclusive) */
  dateEnd?: string; // ISO 8601 format
  /** Filter by bookmark status */
  bookmarked?: boolean;
}

/**
 * A search result with matching conversation info.
 */
export interface SearchResult {
  /** ID of the matching conversation */
  conversationId: string;
  /** Context snippet around the match (50 chars before/after) */
  snippet: string;
  /** Number of matches in this conversation */
  matchCount: number;
  /** Search relevance rank (lower is better) */
  rank: number;
}

/**
 * Project information for the project filter.
 */
export interface ProjectInfo {
  /** Full project path */
  projectPath: string;
  /** Display name */
  projectName: string;
  /** Number of conversations in this project */
  conversationCount: number;
  /** Timestamp of most recent activity */
  lastActivity: string; // ISO 8601 format
}

/**
 * Payload for the conversations-updated Tauri event.
 * Emitted by the file watcher when conversations change.
 */
export interface ConversationsUpdatedEvent {
  /** Number of new conversations added */
  newCount: number;
  /** Number of existing conversations updated */
  updatedCount: number;
  /** Whether this was triggered by file watcher (vs initial load) */
  fromWatcher: boolean;
}
