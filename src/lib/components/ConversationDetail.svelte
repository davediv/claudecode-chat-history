<script lang="ts">
  /**
   * Conversation detail component displaying full conversation content.
   *
   * Features:
   * - Header with project name, date, message count
   * - Messages in chronological order with role distinction
   * - Smooth scrolling for long conversations
   * - Back button for narrow screens
   * - Tag management with autocomplete
   */
  import type { Conversation, TagInfo } from "$lib/types";
  import MessageBubble from "./MessageBubble.svelte";
  import TagInput from "./TagInput.svelte";
  import { exportConversation } from "$lib/services/export";
  import { toast } from "$lib/stores/toast.svelte";

  interface Props {
    /** The conversation to display */
    conversation: Conversation;
    /** Handler for back button (mobile/narrow view) */
    onBack?: () => void;
    /** Handler for bookmark toggle */
    onToggleBookmark?: (id: string) => void;
    /** Handler for tags change */
    onTagsChange?: (id: string, tags: string[]) => void;
    /** All available tags for autocomplete */
    allTags?: TagInfo[];
    /** Handler for navigating to a subagent conversation */
    onSelectSubagent?: (id: string) => void;
  }

  let {
    conversation,
    onBack,
    onToggleBookmark,
    onTagsChange,
    allTags = [],
    onSelectSubagent,
  }: Props = $props();

  let subagentsExpanded = $state(false);

  function handleTagsChange(tags: string[]) {
    onTagsChange?.(conversation.id, tags);
  }

  function handleBookmarkClick() {
    onToggleBookmark?.(conversation.id);
  }

  function handleBookmarkKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onToggleBookmark?.(conversation.id);
    }
  }

  let isExporting = $state(false);

  async function handleExport() {
    if (isExporting) return;
    isExporting = true;
    try {
      const success = await exportConversation(conversation);
      if (success) {
        toast.success("Conversation exported successfully");
      }
      // User cancellation (success === false) doesn't need a toast
    } catch (error) {
      console.error("Export failed:", error);
      toast.error("Failed to export conversation");
    } finally {
      isExporting = false;
    }
  }

  function handleExportKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleExport();
    }
  }

  /**
   * Format a number with compact notation for display.
   * e.g., 1000 -> "1K", 1500000 -> "1.5M"
   */
  function formatTokenCount(count: number): string {
    if (count < 1000) {
      return count.toLocaleString();
    } else if (count < 1000000) {
      return (count / 1000).toFixed(count < 10000 ? 1 : 0) + "K";
    } else {
      return (count / 1000000).toFixed(1) + "M";
    }
  }

  /**
   * Format a date as relative or absolute depending on recency.
   */
  function formatDate(isoString: string): string {
    try {
      const date = new Date(isoString);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

      if (diffDays === 0) {
        return (
          "Today at " +
          date.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
          })
        );
      } else if (diffDays === 1) {
        return (
          "Yesterday at " +
          date.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
          })
        );
      } else if (diffDays < 7) {
        return (
          date.toLocaleDateString("en-US", { weekday: "long" }) +
          " at " +
          date.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
          })
        );
      } else {
        return date.toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
          year: "numeric",
          hour: "numeric",
          minute: "2-digit",
        });
      }
    } catch {
      return "";
    }
  }

  /**
   * Format a short time string for subagent list.
   */
  function formatShortTime(isoString: string): string {
    try {
      const date = new Date(isoString);
      return date.toLocaleTimeString("en-US", {
        hour: "numeric",
        minute: "2-digit",
      });
    } catch {
      return "";
    }
  }

  function toggleSubagents() {
    subagentsExpanded = !subagentsExpanded;
  }

  function handleSubagentClick(id: string) {
    onSelectSubagent?.(id);
  }

  function handleSubagentKeydown(event: KeyboardEvent, id: string) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelectSubagent?.(id);
    }
  }
</script>

<div class="conversation-detail">
  <header class="detail-header">
    <button class="back-button" onclick={() => onBack?.()} aria-label="Back to conversation list">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M15 18l-6-6 6-6"></path>
      </svg>
    </button>

    <div class="header-content">
      <h1 class="project-name" title={conversation.projectName}>{conversation.projectName}</h1>
      <div class="header-meta">
        <span class="meta-date">{formatDate(conversation.lastTime)}</span>
        <span class="meta-separator">•</span>
        <span class="meta-count">
          {conversation.messages.length}
          {conversation.messages.length === 1 ? "message" : "messages"}
        </span>
        {#if conversation.totalTokens && (conversation.totalTokens.input > 0 || conversation.totalTokens.output > 0)}
          <span class="meta-separator">•</span>
          <span class="meta-tokens" title="Input tokens / Output tokens">
            <svg
              class="token-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10"></circle>
              <path d="M12 6v6l4 2"></path>
            </svg>
            {formatTokenCount(conversation.totalTokens.input)} / {formatTokenCount(
              conversation.totalTokens.output
            )}
          </span>
        {/if}
      </div>
    </div>

    <button
      class="action-button export-button"
      onclick={handleExport}
      onkeydown={handleExportKeydown}
      disabled={isExporting}
      aria-label="Export conversation as Markdown"
      title="Export as Markdown"
    >
      {#if isExporting}
        <svg
          class="action-icon spin"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 12a9 9 0 11-6.219-8.56"></path>
        </svg>
      {:else}
        <svg
          class="action-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
      {/if}
    </button>

    <button
      class="action-button bookmark-button"
      class:bookmarked={conversation.bookmarked}
      onclick={handleBookmarkClick}
      onkeydown={handleBookmarkKeydown}
      aria-label={conversation.bookmarked ? "Remove bookmark" : "Bookmark conversation"}
      aria-pressed={conversation.bookmarked}
      title={conversation.bookmarked ? "Remove bookmark" : "Bookmark conversation"}
    >
      <svg
        class="bookmark-icon"
        viewBox="0 0 24 24"
        fill={conversation.bookmarked ? "currentColor" : "none"}
        stroke="currentColor"
        stroke-width="2"
      >
        <path
          d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
        ></path>
      </svg>
    </button>
  </header>

  <div class="tags-section">
    <TagInput tags={conversation.tags ?? []} {allTags} onTagsChange={handleTagsChange} />
  </div>

  {#if conversation.subagents && conversation.subagents.length > 0}
    <div class="subagents-section">
      <button class="subagents-toggle" onclick={toggleSubagents} aria-expanded={subagentsExpanded}>
        <svg
          class="toggle-icon"
          class:expanded={subagentsExpanded}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
        <svg
          class="subagent-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="8" r="4"></circle>
          <path d="M6 20v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2"></path>
        </svg>
        <span class="subagents-label">
          {conversation.subagents.length} subagent{conversation.subagents.length !== 1 ? "s" : ""}
        </span>
      </button>
      {#if subagentsExpanded}
        <div class="subagents-list">
          {#each conversation.subagents as subagent (subagent.id)}
            <div
              class="subagent-item"
              role="button"
              tabindex="0"
              onclick={() => handleSubagentClick(subagent.id)}
              onkeydown={(e) => handleSubagentKeydown(e, subagent.id)}
            >
              <div class="subagent-header">
                <span class="subagent-id">{subagent.agentId}</span>
                <span class="subagent-time">{formatShortTime(subagent.startTime)}</span>
              </div>
              <p class="subagent-preview">{subagent.preview || "No preview"}</p>
              <span class="subagent-meta">{subagent.messageCount} messages</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <div class="messages-container">
    {#each conversation.messages as message (message.id)}
      <MessageBubble {message} />
    {/each}
  </div>
</div>

<style>
  .conversation-detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .back-button {
    display: none;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    background: none;
    border: none;
    border-radius: 4px;
    color: var(--color-text-secondary);
    transition: background-color 0.15s ease;
  }

  .back-button:hover {
    background-color: var(--color-bg-tertiary);
  }

  .back-button svg {
    width: 1.25rem;
    height: 1.25rem;
  }

  /* Show back button on narrow screens */
  @media (max-width: 640px) {
    .back-button {
      display: flex;
    }
  }

  .header-content {
    flex: 1;
    min-width: 0;
  }

  .project-name {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.125rem;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .meta-separator {
    opacity: 0.5;
  }

  .meta-tokens {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .token-icon {
    width: 0.875rem;
    height: 0.875rem;
    opacity: 0.7;
  }

  .tags-section {
    padding: 0.5rem 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .subagents-section {
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .subagents-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 1rem;
    background: none;
    border: none;
    color: var(--color-accent);
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.15s ease;
  }

  .subagents-toggle:hover {
    background-color: var(--color-bg-tertiary);
  }

  .subagents-toggle:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .toggle-icon {
    width: 1rem;
    height: 1rem;
    transition: transform 0.15s ease;
  }

  .toggle-icon.expanded {
    transform: rotate(90deg);
  }

  .subagent-icon {
    width: 1rem;
    height: 1rem;
  }

  .subagents-label {
    flex: 1;
  }

  .subagents-list {
    padding: 0 1rem 0.5rem 2.5rem;
  }

  .subagent-item {
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.25rem;
    background-color: var(--color-bg-tertiary);
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
    outline: none;
  }

  .subagent-item:hover {
    background-color: color-mix(in srgb, var(--color-bg-tertiary) 80%, var(--color-accent) 20%);
  }

  .subagent-item:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .subagent-item:last-child {
    margin-bottom: 0;
  }

  .subagent-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .subagent-id {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-accent);
  }

  .subagent-time {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
  }

  .subagent-preview {
    margin: 0 0 0.25rem 0;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .subagent-meta {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    scroll-behavior: smooth;
  }

  /* Shared action button styles */
  .action-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      color 0.15s ease,
      background-color 0.15s ease;
    flex-shrink: 0;
  }

  .action-button:hover {
    color: var(--color-text-secondary);
    background-color: var(--color-bg-tertiary);
  }

  .action-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .action-button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .action-icon {
    width: 1.125rem;
    height: 1.125rem;
  }

  /* Export button spinner */
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .action-icon.spin {
    animation: spin 1s linear infinite;
  }

  /* Bookmark button active state */
  .bookmark-button.bookmarked {
    color: var(--color-accent);
  }

  .bookmark-button.bookmarked:hover {
    color: var(--color-accent);
    background-color: var(--color-bg-tertiary);
  }

  .bookmark-icon {
    width: 1.125rem;
    height: 1.125rem;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .messages-container {
      scroll-behavior: auto;
    }

    .action-button {
      transition: none;
    }

    .action-icon.spin {
      animation: none;
    }

    .toggle-icon {
      transition: none;
    }

    .subagents-toggle,
    .subagent-item {
      transition: none;
    }
  }
</style>
