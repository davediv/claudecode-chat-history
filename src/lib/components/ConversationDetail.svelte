<script lang="ts">
  /**
   * Conversation detail component displaying full conversation content.
   *
   * Features:
   * - Header with project name, date, message count
   * - Messages in chronological order with role distinction
   * - Smooth scrolling for long conversations
   * - Back button for narrow screens
   */
  import type { Conversation } from "$lib/types";
  import MessageBubble from "./MessageBubble.svelte";

  interface Props {
    /** The conversation to display */
    conversation: Conversation;
    /** Handler for back button (mobile/narrow view) */
    onBack?: () => void;
    /** Handler for bookmark toggle */
    onToggleBookmark?: (id: string) => void;
  }

  let { conversation, onBack, onToggleBookmark }: Props = $props();

  function handleBookmarkClick() {
    onToggleBookmark?.(conversation.id);
  }

  function handleBookmarkKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onToggleBookmark?.(conversation.id);
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
      </div>
    </div>

    <button
      class="bookmark-button"
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

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    scroll-behavior: smooth;
  }

  .bookmark-button {
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

  .bookmark-button:hover {
    color: var(--color-text-secondary);
    background-color: var(--color-bg-tertiary);
  }

  .bookmark-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

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

    .bookmark-button {
      transition: none;
    }
  }
</style>
