<script lang="ts">
  /**
   * Sidebar component for displaying conversation list.
   *
   * Features:
   * - Scrollable list of conversation summaries
   * - Selected state highlighting
   * - Empty state when no conversations
   */

  interface ConversationItem {
    id: string;
    projectName: string;
    preview: string;
    lastTime: string;
    messageCount: number;
  }

  interface Props {
    /** List of conversations to display */
    conversations?: ConversationItem[];
    /** Currently selected conversation ID */
    selectedId?: string | null;
    /** Handler for conversation selection */
    onSelect?: (id: string) => void;
    /** Whether data is loading */
    isLoading?: boolean;
  }

  let { conversations = [], selectedId = null, onSelect, isLoading = false }: Props = $props();

  function formatDate(isoString: string): string {
    try {
      const date = new Date(isoString);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

      if (diffDays === 0) {
        return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
      } else if (diffDays === 1) {
        return "Yesterday";
      } else if (diffDays < 7) {
        return date.toLocaleDateString([], { weekday: "short" });
      } else {
        return date.toLocaleDateString([], { month: "short", day: "numeric" });
      }
    } catch {
      return "";
    }
  }

  function handleKeydown(event: KeyboardEvent, id: string) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect?.(id);
    }
  }
</script>

<aside class="sidebar">
  {#if isLoading}
    <div class="sidebar-loading">
      <div class="loading-spinner"></div>
      <span>Loading conversations...</span>
    </div>
  {:else if conversations.length === 0}
    <div class="sidebar-empty">
      <svg
        class="empty-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path
          d="M20 13V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7m16 0v5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-5m16 0h-2.586a1 1 0 0 0-.707.293l-2.414 2.414a1 1 0 0 1-.707.293h-3.172a1 1 0 0 1-.707-.293l-2.414-2.414A1 1 0 0 0 6.586 13H4"
        ></path>
      </svg>
      <p>No conversations found</p>
      <p class="empty-hint">Conversations will appear here once Claude Code creates them</p>
    </div>
  {:else}
    <ul class="conversation-list" role="listbox" aria-label="Conversations">
      {#each conversations as conversation (conversation.id)}
        <li
          class="conversation-item"
          class:selected={selectedId === conversation.id}
          role="option"
          aria-selected={selectedId === conversation.id}
          tabindex="0"
          onclick={() => onSelect?.(conversation.id)}
          onkeydown={(e) => handleKeydown(e, conversation.id)}
        >
          <div class="conversation-header">
            <span class="project-name">{conversation.projectName}</span>
            <span class="timestamp">{formatDate(conversation.lastTime)}</span>
          </div>
          <p class="preview">{conversation.preview}</p>
          <div class="conversation-meta">
            <span class="message-count">{conversation.messageCount} messages</span>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background-color: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
  }

  .sidebar-loading,
  .sidebar-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
  }

  .loading-spinner {
    width: 2rem;
    height: 2rem;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-icon {
    width: 3rem;
    height: 3rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  .sidebar-empty p {
    margin: 0;
  }

  .empty-hint {
    font-size: 0.75rem;
    margin-top: 0.5rem !important;
  }

  .conversation-list {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
  }

  .conversation-item {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .conversation-item:hover {
    background-color: var(--color-bg-tertiary);
  }

  .conversation-item.selected {
    background-color: var(--color-bg-tertiary);
    border-left: 3px solid var(--color-accent);
    padding-left: calc(1rem - 3px);
  }

  .conversation-item:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .conversation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .project-name {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 70%;
  }

  .timestamp {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .preview {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
  }

  .conversation-meta {
    display: flex;
    gap: 0.75rem;
    margin-top: 0.375rem;
  }

  .message-count {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
  }

  /* Responsive: hide sidebar on narrow screens */
  @media (max-width: 640px) {
    .sidebar {
      position: absolute;
      left: 0;
      top: var(--header-height);
      bottom: 0;
      z-index: 10;
      transform: translateX(-100%);
      transition: transform 0.2s ease;
    }
  }
</style>
