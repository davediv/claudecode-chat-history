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
  import type { Conversation, Message } from "$lib/types";

  interface Props {
    /** The conversation to display */
    conversation: Conversation;
    /** Handler for back button (mobile/narrow view) */
    onBack?: () => void;
  }

  let { conversation, onBack }: Props = $props();

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
   * Format message timestamp for display.
   */
  function formatMessageTime(isoString: string): string {
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

  /**
   * Get the display content from a message's content blocks.
   */
  function getMessageContent(message: Message): string {
    return message.content
      .filter((block) => block.type === "text")
      .map((block) => block.content)
      .join("\n\n");
  }

  /**
   * Get role display label.
   */
  function getRoleLabel(role: Message["role"]): string {
    switch (role) {
      case "user":
        return "You";
      case "assistant":
        return "Claude";
      case "system":
        return "System";
      default:
        return role;
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
  </header>

  <div class="messages-container">
    {#each conversation.messages as message (message.id)}
      <article
        class="message message-{message.role}"
        aria-label="{getRoleLabel(message.role)} message"
      >
        <div class="message-header">
          <span class="message-role">{getRoleLabel(message.role)}</span>
          <span class="message-time">{formatMessageTime(message.timestamp)}</span>
        </div>
        <div class="message-content">
          {getMessageContent(message)}
        </div>
      </article>
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

  .message {
    max-width: 85%;
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    animation: messageSlideIn 0.2s ease-out;
  }

  @keyframes messageSlideIn {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message-user {
    margin-left: auto;
    background-color: var(--color-accent);
    color: white;
  }

  .message-user .message-role,
  .message-user .message-time {
    color: rgba(255, 255, 255, 0.8);
  }

  .message-assistant {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .message-system {
    max-width: 100%;
    margin-left: auto;
    margin-right: auto;
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.8125rem;
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.375rem;
    font-size: 0.75rem;
  }

  .message-role {
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .message-assistant .message-role {
    color: var(--color-accent);
  }

  .message-time {
    color: var(--color-text-muted);
    font-size: 0.6875rem;
  }

  .message-content {
    font-size: 0.875rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .message {
      animation: none;
    }

    .messages-container {
      scroll-behavior: auto;
    }
  }
</style>
